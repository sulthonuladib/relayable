//! `smoke-driver` — test-support client for `scripts/smoke.sh` (`full.md` §13.4).
//!
//! The frozen `relayable` CLI has no send/attach/capture command
//! (`protocol/compatibility/core-001-cli-json.md`: those are deferred), so the
//! smoke test needs a real client to drive terminal input/output. This binary
//! uses only the real `relayable_client` library over the real Unix socket and
//! drives the real server. It is test-support code and owns no production
//! behavior.
//!
//! Subcommands:
//!
//! * `establish --socket <path> --block <id> --nonce <n>` — attach the block as
//!   the writer, set `SMOKE_VAR=relayable-<n>`, print the marker back, verify the
//!   marker round-tripped, then **drop the client** (abrupt disconnect, the
//!   §13.4 "disconnect client" step). It does not detach or close the session.
//! * `resume --socket <path> --block <id> --nonce <n>` — connect a *new* client,
//!   attach the same block, and read back `$SMOKE_VAR`. Seeing the marker proves
//!   the server-owned process is the same shell that survived the disconnect.
//! * `field <json-file> <field-path>` — print one string/number field of a JSON
//!   document, addressed by a dotted path where a numeric component indexes an
//!   array (`session_id`, `session.state`, `blocks.0.program_pid`); exits
//!   non-zero on a missing, empty, or non-scalar field.
//!
//! Every wait is bounded and every failure prints a clear stderr diagnostic and
//! exits non-zero. No production crate is modified.

use std::io::Write as _;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use relayable_client::{AttachMode, Client, ClientConfig, ClientError, ErrorCode};
use relayable_model::BlockId;

/// Bounded wait for a writer lease release / terminal output.
const OUTPUT_WAIT: Duration = Duration::from_secs(10);
/// Bounded retry window while a previous writer lease is being released.
const LEASE_RETRY: Duration = Duration::from_secs(5);

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("smoke-driver: error: {error:#}");
            let _ = std::io::stderr().flush();
            ExitCode::from(1)
        }
    }
}

fn run(args: &[String]) -> Result<()> {
    let command = args
        .first()
        .map(String::as_str)
        .ok_or_else(|| anyhow!("missing subcommand (expected establish|resume|field)"))?;
    match command {
        "establish" => establish(&BlockArgs::parse(&args[1..])?),
        "resume" => resume(&BlockArgs::parse(&args[1..])?),
        "field" => field(&args[1..]),
        other => bail!("unknown subcommand {other:?} (expected establish|resume|field)"),
    }
}

/// `--socket/--block/--nonce` options shared by `establish` and `resume`.
struct BlockArgs {
    socket: std::path::PathBuf,
    block: BlockId,
    nonce: String,
}

impl BlockArgs {
    fn parse(rest: &[String]) -> Result<Self> {
        let mut socket: Option<String> = None;
        let mut block: Option<String> = None;
        let mut nonce: Option<String> = None;
        let mut index = 0;
        while index < rest.len() {
            let key = rest[index].clone();
            let value = rest
                .get(index + 1)
                .ok_or_else(|| anyhow!("missing value after {key}"))?
                .clone();
            match key.as_str() {
                "--socket" => socket = Some(value),
                "--block" => block = Some(value),
                "--nonce" => nonce = Some(value),
                other => bail!("unknown option {other:?}"),
            }
            index += 2;
        }
        let block = block
            .ok_or_else(|| anyhow!("--block is required"))?
            .parse::<BlockId>()
            .context("--block is not a Relayable block id (`block:<uuid>`)")?;
        let nonce = nonce.ok_or_else(|| anyhow!("--nonce is required"))?;
        if nonce.is_empty() {
            bail!("--nonce must not be empty");
        }
        Ok(Self {
            socket: std::path::PathBuf::from(
                socket.ok_or_else(|| anyhow!("--socket is required"))?,
            ),
            block,
            nonce,
        })
    }
}

/// Connect, set the marker variable, prove the marker round-trips, then drop the
/// client without detaching — the required "disconnect the client" step.
fn establish(args: &BlockArgs) -> Result<()> {
    let mut client = connect(args)?;
    let stream = attach_writer(&mut client, &args.block)?;
    let marker = format!("relayable-{}", args.nonce);
    // `SMOKE_SET:%s` in the echoed command cannot match the expanded marker, so
    // the wait below can only succeed on real command output.
    let script = format!("SMOKE_VAR={marker}\nprintf 'SMOKE_SET:%s\\n' \"$SMOKE_VAR\"\n");
    client
        .send_input(stream, script.as_bytes())
        .context("terminal.input (establish)")?;
    let needle = format!("SMOKE_SET:{marker}");
    // `wait_for_output` returns `Ok` only once the needle is present in the
    // accumulated output, so a separate `contains` check would assert nothing.
    let _output = client
        .wait_for_output(stream, needle.as_bytes(), OUTPUT_WAIT)
        .with_context(|| format!("waiting for {needle}"))?;
    println!("[smoke-driver] establish OK: SMOKE_VAR={marker} round-tripped");
    // Abrupt disconnect: no `block.detach`, no `session.close`. The server-owned
    // shell must survive this drop (§13.4).
    drop(client);
    Ok(())
}

/// Connect a new client and read back the marker variable from the same shell.
fn resume(args: &BlockArgs) -> Result<()> {
    let mut client = connect(args)?;
    let stream = attach_writer(&mut client, &args.block)?;
    let marker = format!("relayable-{}", args.nonce);
    let script = "printf 'SMOKE_CONT:%s\\n' \"$SMOKE_VAR\"\n";
    client
        .send_input(stream, script.as_bytes())
        .context("terminal.input (resume)")?;
    let needle = format!("SMOKE_CONT:{marker}");
    // `wait_for_output` returns `Ok` only once the needle is present, so no
    // extra `contains` check is needed here.
    let _output = client
        .wait_for_output(stream, needle.as_bytes(), OUTPUT_WAIT)
        .with_context(|| format!("waiting for {needle}"))?;
    println!("[smoke-driver] resume OK: same shell still holds SMOKE_VAR={marker}");
    // A detach failure is a real failure: the smoke path must not silently
    // ignore it (the reconnect proof above is only valid on a clean detach).
    client
        .detach_block(stream)
        .context("block.detach after resume")?;
    Ok(())
}

fn connect(args: &BlockArgs) -> Result<Client> {
    Client::connect(&args.socket, ClientConfig::new("smoke-driver"))
        .with_context(|| format!("connecting to {}", args.socket.display()))
}

/// Attach the block as the writer, retrying while a stale lease is released.
fn attach_writer(client: &mut Client, block: &BlockId) -> Result<u64> {
    let deadline = Instant::now() + LEASE_RETRY;
    loop {
        match client.attach_block(block, AttachMode::Writer) {
            Ok(outcome) => return Ok(outcome.stream_id),
            Err(ClientError::Server(ref error))
                if error.code == ErrorCode::InputNotOwned as i32 && Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(error) => {
                return Err(anyhow::Error::new(error).context("block.attach as Writer"));
            }
        }
    }
}

/// Print one scalar field of a JSON file. The field is addressed by a dotted
/// path where a numeric component indexes an array: `session.state`,
/// `blocks.0.program_pid`. Fails loudly on any other shape.
fn field(args: &[String]) -> Result<()> {
    if args.len() != 2 {
        bail!("field expects exactly <json-file> <field-path>");
    }
    let path = &args[0];
    let selector = &args[1];
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {path}"))?;
    let value: serde_json::Value =
        serde_json::from_str(&text).with_context(|| format!("{path} is not valid JSON"))?;
    let mut field = &value;
    for component in selector.split('.') {
        if component.is_empty() {
            bail!("{path} field path {selector:?} has an empty component");
        }
        field = match field {
            serde_json::Value::Object(map) => map
                .get(component)
                .ok_or_else(|| anyhow!("{path} has no field {component:?} (path {selector:?})"))?,
            serde_json::Value::Array(list) => {
                let index: usize = component.parse().with_context(|| {
                    format!("{path} field path {selector:?}: {component:?} is not an array index")
                })?;
                list.get(index).ok_or_else(|| {
                    anyhow!("{path} field path {selector:?}: array index {index} out of range")
                })?
            }
            other => {
                bail!("{path} field path {selector:?}: cannot index {other} with {component:?}")
            }
        };
    }
    match field {
        serde_json::Value::String(text) if !text.is_empty() => {
            println!("{text}");
            Ok(())
        }
        serde_json::Value::Number(number) => {
            println!("{number}");
            Ok(())
        }
        serde_json::Value::String(_) => bail!("field {selector:?} is an empty string"),
        other => bail!("field {selector:?} is not a string or number: {other}"),
    }
}
