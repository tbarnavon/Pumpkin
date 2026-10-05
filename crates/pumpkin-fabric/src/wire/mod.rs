//! Byte layouts of the Fabric payloads Pumpkin speaks.
//!
//! Everything version-sensitive about Fabric's protocol lives in this module, one file per
//! payload, each citing the Java it mirrors (fabric-api, branch 1.21.1, 0.116.17). Updating to a new
//! Fabric API means re-reading those files. The `FriendlyByteBuf` helpers here are shared with
//! the other loaders' crates; `minecraft:register` and the `c:` channels are common to them.

pub mod common;
pub mod register;
pub mod registry_sync;

/// Why a payload from the client could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ReadError {
    #[error("payload ended early")]
    UnexpectedEnd,
    #[error("VarInt is too long")]
    VarIntTooLong,
    #[error("length {0} is out of bounds")]
    BadLength(i32),
    #[error("string is not UTF-8")]
    InvalidUtf8,
}

/// Longest string `FriendlyByteBuf.readUtf()` accepts by default (`Utf8String.MAX_LENGTH`).
pub const MAX_STRING_CHARS: usize = 32767;

pub fn write_var_int(out: &mut Vec<u8>, value: i32) {
    let mut value = value as u32;
    loop {
        let byte = (value & 0x7F) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

/// `FriendlyByteBuf.writeUtf`: `VarInt` byte length, then UTF-8.
pub fn write_string(out: &mut Vec<u8>, value: &str) {
    write_var_int(out, value.len() as i32);
    out.extend_from_slice(value.as_bytes());
}

/// Bounded reader over a payload body.
pub struct Reader<'a> {
    data: &'a [u8],
}

impl<'a> Reader<'a> {
    #[must_use]
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn byte(&mut self) -> Result<u8, ReadError> {
        let (&first, rest) = self.data.split_first().ok_or(ReadError::UnexpectedEnd)?;
        self.data = rest;
        Ok(first)
    }

    pub fn var_int(&mut self) -> Result<i32, ReadError> {
        let mut value = 0u32;
        for shift in (0..35).step_by(7) {
            let byte = self.byte()?;
            value |= u32::from(byte & 0x7F) << shift;
            if byte & 0x80 == 0 {
                return Ok(value as i32);
            }
        }
        Err(ReadError::VarIntTooLong)
    }

    /// A `VarInt` length or count, rejected if negative or above `max`.
    pub fn length(&mut self, max: usize) -> Result<usize, ReadError> {
        let raw = self.var_int()?;
        usize::try_from(raw)
            .ok()
            .filter(|len| *len <= max)
            .ok_or(ReadError::BadLength(raw))
    }

    pub const fn bytes(&mut self, len: usize) -> Result<&'a [u8], ReadError> {
        if len > self.data.len() {
            return Err(ReadError::UnexpectedEnd);
        }
        let (head, rest) = self.data.split_at(len);
        self.data = rest;
        Ok(head)
    }

    /// `FriendlyByteBuf.readUtf()`: at most 32767 characters, so at most 3 bytes per char.
    pub fn string(&mut self) -> Result<&'a str, ReadError> {
        let len = self.length(MAX_STRING_CHARS * 3)?;
        std::str::from_utf8(self.bytes(len)?).map_err(|_| ReadError::InvalidUtf8)
    }

    #[must_use]
    pub const fn rest(&self) -> &'a [u8] {
        self.data
    }
}
