# UI-004 stage 1 — auto-session invariant + truthful empty state (DEC-019)

**Task:** `UI-004` stage 1, approved as DEC-019 stage 1. Make "the app ends up with a
session" true without a manual `new`, and make the empty surface say what is really
happening.
**Worktree:** `/home/sulthonuladib/work/relayable-wt/ui-004-stage1`
**Branch:** `ui/ui-004-stage1-auto-session` (from `master` `277bc23`).
**Scope:** `apps/linux/src/app/{events,tabs,panes,helpers,tests}.rs` only, plus this
notes file and `ui-004-run.log`. Nothing under `src/api/**`, `src/ui/**`, `src/term/**`,
`src/{geometry*,theme*,tokens}**`, `src/main.rs`, `app/{state,render,resize,keys,picker,top_bar,lifecycle}.rs`,
`Cargo.{toml,lock}`, `crates/**`, `scripts/**`, `docs/**` changed — see the scope proof in
`ui-004-run.log` §8. `apps/linux/src/api/**` belongs to UI-003, which was in flight in a
different worktree; it was **not** touched.

## 1. Files changed

| Path | Lines after | Change |
| --- | --- | --- |
| `apps/linux/src/app/helpers.rs` | 128 | new pure `needs_session`, `empty_state_message`, `abandon_create` |
| `apps/linux/src/app/events.rs` | 224 | invariant on the `Connected` edge, on the data-plane restore edge, and after an error; re-arm on error/disconnect |
| `apps/linux/src/app/tabs.rs` | 195 | new `ensure_session` + `abandon_pending_create`; invariant after the last tab closes |
| `apps/linux/src/app/panes.rs` | 324 | empty-surface message now derives from `connection` **and** `pending_create` |
| `apps/linux/src/app/tests.rs` | 354 | 7 new headless tests (58 → 65 in the desktop suite) |

Diff: 367 insertions, 18 deletions, 5 files. All files are well inside the
`docs/architecture/code-organization.md` caps (test cap 1200, prod cap 1000).

## 2. Defect 1 — `tabs.is_empty()` was a one-shot trigger, not an invariant

**Verified:** the `Connected` arm in `events.rs` created a session only if
`self.tabs.is_empty() && self.pending_create.is_none()`, and `tabs.rs::close_focused`
removed the last tab and returned (`self.active_tab = 0; return;`) without re-checking.
The `Connected` edge does not come again, so the app sat on the empty surface forever.

**Fixed:** the decision is now the pure helper

```rust
needs_session(connection, tabs_empty, create_in_flight)
    = tabs_empty && !create_in_flight && matches!(connection, ConnectionState::Connected)
```

and the action is `WorkspaceView::ensure_session()` (`tabs.rs`), re-checked from every
point where the app can be connected with zero tabs:

1. the `Connected` edge (`events.rs`);
2. after the last tab closes (`close_focused`, `tabs.rs`);
3. after a failed create (the `ApiEvent::Error` arm, `events.rs`) — see §3;
4. the `data_plane_connected` restore path (`events.rs`), which also sets the app state
   back to `Connected`. This was **not** in the brief; without it that path can restore
   `Connected` with zero tabs and no create in flight, which is exactly the state the
   invariant forbids. It matters because the data-plane reconnect happens while panes
   exist, but the same arm also runs for an app that has no tabs yet.

`ensure_session` cannot duplicate a request: `request_new_tab` → `create_session` records
`pending_create` synchronously in the same event, so repeated `Connected` edges are
no-ops.

## 3. Defect 2 — a failed create wedged the app permanently

**Verified:** `create_session` returns early while `pending_create.is_some()`, and the
`ApiEvent::Error` arm set `self.connection = ConnectionState::Error(..)` without clearing
`pending_create`. One failed `session.create` therefore killed auto-create *and*
`Ctrl+Shift+N` for the rest of the run.

**Fixed:** `abandon_pending_create(cause)` clears the intent, and the rule itself is the
pure helper `abandon_create(pending_create: &mut Option<CreateIntent>) -> bool`
(`helpers.rs`), so it is exercised by the display-free tests. It is called from:

* the `ApiEvent::Error` arm (`events.rs`);
* the `ApiEvent::Disconnected` arm (`events.rs`) — **not in the brief**, but the same
  wedge: a create in flight when the control transport dies is never answered
  (`api/control.rs` keeps draining the same command queue and does **not** replay it after
  a reconnect), so without this the *next* `Connected` edge would find `pending_create`
  still set and never create either. This needs no `api/**` change; it is a statement
  about the frozen `ApiCommand`/`ApiEvent` shapes.

**Retry policy (explicit, and the reason there is no create loop):** re-arm on failure,
then re-check the invariant, but the invariant requires the app's own
`ConnectionState::Connected`. Both failure arms set `Error`/`Disconnected` first, so the
re-check is a deliberate no-op and one failed create can never spin. Retries are bounded
to **one auto-create per `Connected` edge**, plus the manual `Ctrl+Shift+N`, which works
again immediately because the in-flight flag is clear. A bounded *immediate* retry (e.g.
"try twice, then give up") would need a new counter field in `app.rs` — not an owned path
— and would still need a way to tell a rejected create from a transport failure. It is
therefore recorded as stage 2, not invented here.

## 4. Defect 3 — retryable connect failure shown as a terminal error: **left alone**

Not fixed, as instructed. The classification ("connect attempt N failed, still retrying"
vs. terminal) lives in `apps/linux/src/api/control.rs`, owned by UI-003. `app/**` did not
gain any message string matching or new `ApiEvent` variant; the pre-existing
`message.starts_with("block.attach")` FIFO fix is untouched. **Stage-2 item (needs
`api/**`):** `ApiEvent::Error` is currently indistinguishable from a terminal failure, so
the app paints `ConnectionState::Error` — which is also why, after a failed create while
the control connection is actually alive, the app shows the error banner and will not
auto-create again until another `Connected` edge arrives. That is the honest current
behaviour and it is not papered over here. Two follow-ups belong to stage 2 with `api/**`:
(a) a non-terminal error signal, (b) surfacing `session.view`/an attach command so an
existing session can be restored instead of always creating a new one.

## 5. Empty surface copy — all four surfaces are Relayable-original

The research catalogue records each of these as `not observed` in all five reference
videos (`docs/research/ui-catalog.md` — e.g. lines 78–79, 145–146, 209–211, 243–244,
388–390), for every entry. **The empty, loading, error and disconnected surfaces are
therefore Relayable-original copy, not reference-derived.** They must not be presented as
matching the reference, and no reference fidelity claim is made about them. What the
change does keep faithful is the *shape*: `render_empty_state` still renders the same two
children in the same flex column with the same tokens and geometry; only the message
string selection moved into a pure helper (no new in-flow child, so the measured body
geometry cannot move — the connection banner stays the only in-flow banner). No layout,
geometry, token, styling or render-tree structure changed.

Message selection (`empty_state_message(connection, create_in_flight)`):

| State | Message |
| --- | --- |
| connected, create in flight | `No sessions yet — creating one…` |
| connected, nothing in flight | `No session — Ctrl+Shift+N creates one` |
| connecting | `Connecting to the Relayable server…` |
| reconnecting | `Reconnecting…` |
| disconnected | `Disconnected — no live server connection` |
| error | `Server error — see the banner` |

The idle (connected, not creating) string no longer claims a create is in flight; a test
asserts that it does not contain "creating" and that all six states are distinct. The
reconnect `attempt` counter is deliberately not in the copy — the top bar owns it
(`top_bar.rs`, unchanged), and `Reconnecting { attempt }` renders identically for any
attempt.

## 6. Honest deviation that stage 1 does not close

After this stage the app **still creates a new session on every launch** instead of
restoring the previous one. That is a deviation from the only behavioural evidence in the
reference videos (V1 0:52–1:30, V4 4:11). Restoring an existing session is DEC-019 stage 2
and stays deferred: it needs `api/**` (the wire `SessionSummary` in `session.list` carries
neither a block id nor `created_at`, so a restore cannot be derived from the list) **and**
a user policy decision on *which* session to restore (a locally persisted last-used id is
Relayable-original, not reference-derived). Recorded, not decided, here.

## 7. Tests (7 new; desktop suite 58 → 65, all headless)

`app/tests.rs` is gpui-free, so `WorkspaceView` (which needs an `ApiHandle` and a
`FocusHandle`) cannot be constructed. The new tests drive the **real** helpers
(`needs_session`, `abandon_create`, `empty_state_message`) from a small
`AutoSessionProbe` that mirrors the view's call sites and counts `CreateSession` sends.

| Test | Contract | Mutation it catches (verified) |
| --- | --- | --- |
| `zero_tabs_and_connected_requests_a_session_exactly_once` | one request per `Connected` edge, repeated edges do not stack | M1 (drop `create_in_flight`), M2 (drop `tabs_empty`) |
| `closing_the_last_tab_requests_a_session_again` | last tab closed while connected ⇒ second request, no manual `new` | M2, M3 (drop the `Connected` term) |
| `closing_the_last_tab_while_not_connected_does_not_request_a_session` | offline ⇒ wait for a `Connected` edge | M3 |
| `a_failed_create_re_arms_the_manual_shortcut_and_the_next_connect_edge` | failure clears the flag, does not retry immediately, `Ctrl+Shift+N` works, next edge retries exactly once | M1, M3, M4 (`abandon_create` stops re-arming) |
| `a_disconnect_voids_an_in_flight_create` | a void create must not block the next edge | M4 |
| `the_invariant_never_fires_while_not_connected` | connecting/reconnecting/disconnected/error never start a create | M1, M2, M3 |
| `empty_state_messages_are_distinct_and_truthful` | six distinct truthful strings; idle ≠ "creating one…" | M5 (idle message reuses the creating string) |

The mutation checks were **run for real** (mutate the production helper, run the full
desktop suite, restore) and their verbatim output is in `ui-004-run.log` §3, including a
control run after the restore.

**Honest gap:** a sixth mutation (M6 — deleting the `self.ensure_session()` call from
`close_focused`) is **not** caught by this suite: the display-free tests cover the
*decisions*, not the view's call sites. Wiring and site coverage therefore rely on diff
review, and exactly the same limit applies to the `events.rs` sites. The real-app run in
§7 of the log covers the last-tab-close wiring end-to-end for the mock path.

## 8. What was actually run (full transcript: `ui-004-run.log`)

| Command | Result |
| --- | --- |
| `cd apps/linux && cargo fmt --check` | exit 0 |
| `CARGO_NET_OFFLINE=true cargo clippy --all-targets --locked -- -D warnings` | exit 0 (only the pre-existing transitive `proc-macro-error2` future-incompat note) |
| `CARGO_NET_OFFLINE=true cargo test --locked` | exit 0 — **58 → 65 passed / 0 failed** |
| `CARGO_NET_OFFLINE=true cargo check --locked` | exit 0 |
| `CARGO_NET_OFFLINE=true cargo build --locked` (for the real run) | exit 0 |
| `bash scripts/validate.sh` (root) | exit 0 — all 8 steps + "All CORE-001 validation checks passed." |
| `bash scripts/smoke.sh` | `SMOKE OK`, all 12 steps |
| `bash scripts/check-file-sizes.sh --strict` | exit 0 — `over-hard-cap=0` |
| real app, `--mock --canvas 1099 706`, live Wayland (`wayland-1`) | log shows `relayable: MOCK mode …`, `connection state = Connected`, `no session and no create in flight; requesting one`, `session.created …`; screenshot `922x1030` shows a live pane and the `MOCK — synthetic in-process data; not a live relayable-server` banner |
| `wtype -M ctrl -M shift -k w` on the focused relayable window | second `no session and no create in flight; requesting one` + `session.created …` in the same log — defect 1 proven on the real binary |

`evidence/run-ui-002.sh` was **not** run: it drives the full live capture matrix and would
overwrite tracked UI-002 PNG artefacts (outside this task's owned paths). The bounded mock
run above was used instead, with all artefacts under `/tmp/opencode` so nothing tracked
outside the owned paths changed.

## 9. Not verified / risks

* **Live server path not exercised end-to-end.** The auto-session happy path was run
  against `--mock`; a real `relayable-server` run (create → attach) was not done in this
  task, and no live connect-failure was reproduced. The `Error`/`Disconnected` re-arm paths
  are covered by the headless tests and by reasoning about `api/control.rs`, not by a live
  failure injection.
* **Split-intent race (bounded, documented).** Any `ApiEvent::Error` while a create is in
  flight clears the intent, including an unrelated data-plane error. If that happens while
  a manual split (`Ctrl+Shift+E`) is in flight, the later `SessionCreated` falls back to a
  new **tab** instead of a split pane (`events.rs` uses
  `pending_create.take().unwrap_or(NewTab)`). The window is narrow (the control thread is
  serialized) and the failure is non-fatal, but it is a real behaviour change and is
  recorded here rather than hidden. Distinguishing "this error is the create's reply" needs
  the `api/**` event shape to change — stage 2.
* **`pending_attach` FIFO untouched.** `abandon_create` only clears the create intent; the
  attach FIFO keeps its existing `block.attach` handling.
* The `data_plane_connected` invariant site trusts the pre-existing (unchanged) app-side
  choice to restore `Connected` from that event; if the control plane is in fact down, the
  create that follows fails and is re-armed like any other failure.
* No geometry/layout/token change was made, so no new screenshot fidelity check was
  needed; the one real screenshot was taken only to confirm the app state, not to compare
  against a reference frame.
