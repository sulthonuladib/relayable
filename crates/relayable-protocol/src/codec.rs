//! Length-prefixed frame codec (contract §3).
//!
//! Every message is a 4-byte little-endian unsigned length prefix followed by
//! one serialized [`Envelope`](crate::v1::Envelope) body. The same framing is
//! used over the local Unix socket and over the remote transport.

use std::io::{Read, Write};

use prost::Message;

use crate::v1::Envelope;

/// Failure modes of the frame codec. Oversized, truncated, zero-length, and
/// malformed-body frames are all distinguishable by variant so a caller can
/// answer with a precise structured error instead of failing silently.
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    /// Underlying transport read/write failure.
    #[error("frame transport i/o error: {0}")]
    Io(#[from] std::io::Error),
    /// A frame prefix declared more bytes than were available before EOF.
    #[error("truncated frame body")]
    Truncated,
    /// A frame declared more bytes than `max_frame_bytes` allows.
    #[error("frame of {len} bytes exceeds the {max}-byte limit")]
    FrameTooLarge {
        /// Declared frame length.
        len: usize,
        /// Configured maximum accepted frame length.
        max: u32,
    },
    /// A frame declared a zero-byte body, which is never a valid envelope.
    #[error("zero-length frame")]
    ZeroLengthFrame,
    /// The frame body did not decode as an envelope.
    #[error("malformed envelope body: {0}")]
    Decode(#[from] prost::DecodeError),
}

/// Write one length-prefixed envelope to `w`.
///
/// # Errors
///
/// Returns [`CodecError::ZeroLengthFrame`] if `env` serializes to an empty body,
/// [`CodecError::FrameTooLarge`] if it does not fit a `u32` length, or
/// [`CodecError::Io`] on a transport failure.
pub fn write_envelope<W: Write>(w: &mut W, env: &Envelope) -> Result<(), CodecError> {
    let body = env.encode_to_vec();
    if body.is_empty() {
        return Err(CodecError::ZeroLengthFrame);
    }
    let len = u32::try_from(body.len()).map_err(|_| CodecError::FrameTooLarge {
        len: body.len(),
        max: u32::MAX,
    })?;
    w.write_all(&len.to_le_bytes())?;
    w.write_all(&body)?;
    Ok(())
}

/// Read one length-prefixed envelope from `r`.
///
/// Returns `Ok(None)` on a clean EOF exactly at a frame boundary. A partial
/// prefix or body is [`CodecError::Truncated`]; an over-limit or zero-length
/// prefix is rejected before any body is buffered.
///
/// # Errors
///
/// See [`CodecError`].
pub fn read_envelope<R: Read>(
    r: &mut R,
    max_frame_bytes: u32,
) -> Result<Option<Envelope>, CodecError> {
    let mut prefix = [0u8; 4];
    if !read_full_prefix(r, &mut prefix)? {
        return Ok(None);
    }
    let len = u32::from_le_bytes(prefix) as usize;
    if len == 0 {
        return Err(CodecError::ZeroLengthFrame);
    }
    if len > max_frame_bytes as usize {
        return Err(CodecError::FrameTooLarge {
            len,
            max: max_frame_bytes,
        });
    }
    let mut body = vec![0u8; len];
    match r.read_exact(&mut body) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            return Err(CodecError::Truncated);
        }
        Err(e) => return Err(CodecError::Io(e)),
    }
    Ok(Some(Envelope::decode(body.as_slice())?))
}

/// Fill `prefix` completely, returning `false` for a clean EOF before any byte.
fn read_full_prefix<R: Read>(r: &mut R, prefix: &mut [u8; 4]) -> Result<bool, CodecError> {
    let mut filled = 0;
    while filled < prefix.len() {
        match r.read(&mut prefix[filled..]) {
            Ok(0) if filled == 0 => return Ok(false),
            Ok(0) => return Err(CodecError::Truncated),
            Ok(n) => filled += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(CodecError::Io(e)),
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::limits::DEFAULT_LIMITS;
    use crate::v1::Heartbeat;

    fn sample() -> Envelope {
        Envelope {
            protocol_version: "1.0".to_string(),
            request_id: Some(7),
            stream_id: 0,
            sequence: Some(11),
            trace_id: Some("trace-1".to_string()),
            timestamp_unix_millis: Some(1_700_000_000_000),
            body: Some(crate::v1::envelope::Body::Heartbeat(Heartbeat {
                sequence: Some(3),
            })),
        }
    }

    #[test]
    fn round_trips_an_envelope() {
        let env = sample();
        let mut buf = Vec::new();
        write_envelope(&mut buf, &env).unwrap();
        // 4-byte prefix + non-empty body.
        assert!(buf.len() > 4);

        let mut cursor = std::io::Cursor::new(buf);
        let decoded = read_envelope(&mut cursor, DEFAULT_LIMITS.max_frame_bytes).unwrap();
        assert_eq!(decoded, Some(env));
    }

    #[test]
    fn clean_eof_yields_none() {
        let mut cursor = std::io::Cursor::new(Vec::<u8>::new());
        assert_eq!(
            read_envelope(&mut cursor, DEFAULT_LIMITS.max_frame_bytes).unwrap(),
            None
        );
    }

    #[test]
    fn zero_length_frame_is_rejected() {
        let mut cursor = std::io::Cursor::new(0u32.to_le_bytes().to_vec());
        assert!(matches!(
            read_envelope(&mut cursor, DEFAULT_LIMITS.max_frame_bytes),
            Err(CodecError::ZeroLengthFrame)
        ));
    }

    #[test]
    fn oversized_frame_is_rejected_before_body_read() {
        let max = 16u32;
        let mut cursor = std::io::Cursor::new((max + 1).to_le_bytes().to_vec());
        match read_envelope(&mut cursor, max) {
            Err(CodecError::FrameTooLarge { len, max: m }) => {
                assert_eq!(len, 17);
                assert_eq!(m, 16);
            }
            other => panic!("expected FrameTooLarge, got {other:?}"),
        }
    }

    #[test]
    fn truncated_body_is_rejected() {
        let mut bytes = 10u32.to_le_bytes().to_vec();
        bytes.extend_from_slice(&[1, 2, 3]);
        let mut cursor = std::io::Cursor::new(bytes);
        assert!(matches!(
            read_envelope(&mut cursor, DEFAULT_LIMITS.max_frame_bytes),
            Err(CodecError::Truncated)
        ));
    }

    #[test]
    fn truncated_prefix_is_rejected() {
        let mut cursor = std::io::Cursor::new(vec![1u8, 2]);
        assert!(matches!(
            read_envelope(&mut cursor, DEFAULT_LIMITS.max_frame_bytes),
            Err(CodecError::Truncated)
        ));
    }

    #[test]
    fn malformed_body_is_decode_error() {
        // A 1-byte body `0xFF` decodes as an unknown field tag start: prost
        // reports a decode error.
        let mut bytes = 1u32.to_le_bytes().to_vec();
        bytes.push(0xFF);
        let mut cursor = std::io::Cursor::new(bytes);
        assert!(matches!(
            read_envelope(&mut cursor, DEFAULT_LIMITS.max_frame_bytes),
            Err(CodecError::Decode(_))
        ));
    }

    #[test]
    fn writing_an_empty_envelope_is_rejected() {
        let mut buf = Vec::new();
        assert!(matches!(
            write_envelope(&mut buf, &Envelope::default()),
            Err(CodecError::ZeroLengthFrame)
        ));
    }
}
