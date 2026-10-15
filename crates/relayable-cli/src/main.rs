//! Relayable command-line entry point.
//!
//! The binary is `relayable` (`full.md` §0.1); the `rex` spellings in the
//! reference behavior matrix are reference evidence only and never ship.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(relayable_cli::run(&args));
}
