#![no_std]
#![forbid(unsafe_code)]
#![allow(unused_imports)]

use core::convert::{TryFrom, TryInto};

const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

#[doc(hidden)]
pub fn drop_png_signature(bytes: &[u8]) -> Option<&[u8]> {
  if bytes.len() < 8 {
    None
  } else if &bytes[..8] == &PNG_SIGNATURE {
    Some(&bytes[8..])
  } else {
    None
  }
}

pub struct PngChunkIter<'b> {
  bytes: &'b [u8],
}
impl<'b> PngChunkIter<'b> {
  pub fn from_png_bytes(bytes: &'b [u8]) -> Option<Self> {
    // TODO: test that this returns None in appropriate cases.
    drop_png_signature(bytes).map(|bytes| Self { bytes })
  }
}
impl<'b> core::iter::Iterator for PngChunkIter<'b> {
  type Item = PngChunk<'b>;

  fn next(&mut self) -> Option<PngChunk<'b>> {
    if self.bytes.len() < 12 {
      return None;
    }
    let length = u32::from_be_bytes(self.bytes[0..4].try_into().unwrap());
    let chunk_type = ChunkType(self.bytes[4..8].try_into().unwrap());
    if self.bytes.len() < (length as usize) + 4 {
      return None;
    }
    let chunk_data = &self.bytes[8..(8 + length as usize)];
    let declared_crc = u32::from_be_bytes(self.bytes[(8 + length as usize)..(8 + length as usize + 4)].try_into().unwrap());
    self.bytes = &self.bytes[(8 + length as usize + 4)..];
    Some(PngChunk { length, chunk_type, chunk_data, declared_crc })
  }
}

#[derive(Debug, Copy, Clone)]
pub struct PngChunk<'b> {
  length: u32,
  chunk_type: ChunkType,
  chunk_data: &'b [u8],
  declared_crc: u32,
}
impl<'b> PngChunk<'b> {
  pub fn get_actual_crc(&self) -> u32 {
    const fn make_crc_table() -> [u32; 256] {
      let mut n = 0_usize;
      let mut table = [0_u32; 256];
      while n < 256 {
        let mut c = n as u32;
        let mut k = 0;
        while k < 8 {
          c = if c & 1 != 0 { 0xedb88320 ^ (c >> 1) } else { c >> 1 };
          //
          k += 1;
        }
        table[n] = c;
        //
        n += 1;
      }
      table
    }
    const CRC_TABLE: [u32; 256] = make_crc_table();
    fn update_crc(mut crc: u32, byte_iter: impl Iterator<Item = u8>) -> u32 {
      for b in byte_iter {
        crc = CRC_TABLE[(crc ^ b as u32) as usize & 0xFF] ^ (crc >> 8);
      }
      crc
    }
    fn crc(byte_iter: impl Iterator<Item = u8>) -> u32 {
      update_crc(u32::MAX, byte_iter) ^ u32::MAX
    }
    crc(self.chunk_type.0.iter().copied().chain(self.chunk_data.iter().copied()))
  }
}

#[derive(Copy, Clone)]
struct ChunkType(pub(crate) [u8; 4]);
#[allow(dead_code)]
impl ChunkType {
  pub const fn is_critical(self) -> bool {
    (self.0[0] & 32) > 0
  }
  pub const fn is_public(self) -> bool {
    (self.0[1] & 32) > 0
  }
  pub const fn is_not_reserved(self) -> bool {
    (self.0[2] & 32) > 0
  }
  pub const fn is_trouble_to_copy(self) -> bool {
    (self.0[3] & 32) > 0
  }
}
impl core::fmt::Debug for ChunkType {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let [a, b, c, d] = self.0;
    write!(f, "{}{}{}{}", a as char, b as char, c as char, d as char)
  }
}
