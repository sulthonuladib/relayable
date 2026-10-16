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
mod tests;
