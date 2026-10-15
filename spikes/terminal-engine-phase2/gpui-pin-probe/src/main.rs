//! Trivial GPUI pin probe binary.
//!
//! This exists only to give the `gpui = "=0.2.2"` dependency a real binary
//! target for `cargo check`.  It deliberately does not open a window or touch
//! any GPUI API: display/window proof is out of scope for TERM-002 phase A and
//! is deferred to UI-001.  `main` must stay buildable even if GPUI itself only
//! fails at link time.

fn main() {
    println!("relayable-gpui-pin-probe: gpui =0.2.2 (check-only; no window)");
}
