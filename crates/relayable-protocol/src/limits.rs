//! Provisional protocol limits.
//!
//! These numbers are placeholders, not measurements. The frozen contract
//! deliberately leaves frame size, stream count, credit window, heartbeat
//! interval, and idle timeout open ("What this contract deliberately does not
//! decide"), and `full.md` §12 requires measurement before setting targets.
//! They exist so the server and client can be wired up and then tuned.

use crate::v1::Limits;

/// Default advertised limits.
///
/// All values are provisional configuration parameters pending measurement.
pub const DEFAULT_LIMITS: Limits = Limits {
    max_frame_bytes: 1_048_576,
    max_concurrent_streams: 8,
    credit_window_bytes: 262_144,
    heartbeat_interval_millis: 5_000,
    idle_timeout_millis: 30_000,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_the_documented_provisionals() {
        assert_eq!(DEFAULT_LIMITS.max_frame_bytes, 1_048_576);
        assert_eq!(DEFAULT_LIMITS.max_concurrent_streams, 8);
        assert_eq!(DEFAULT_LIMITS.credit_window_bytes, 262_144);
        assert_eq!(DEFAULT_LIMITS.heartbeat_interval_millis, 5_000);
        assert_eq!(DEFAULT_LIMITS.idle_timeout_millis, 30_000);
    }
}
