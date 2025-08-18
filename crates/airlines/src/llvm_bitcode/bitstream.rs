use anyhow::{Result, anyhow};
use std::io::{Seek, SeekFrom};

use super::{BitCursor, StreamEntry, StreamParser, debug};

#[derive(Debug)]
pub struct Signature {
    magic: u32,
    version: u32,
    offset: u32,
    size: u32,
    cpu_type: u32,
}

pub const BITCODE_WRAPPER_MAGIC: u32 = 0x0b17c0de;

#[derive(Debug)]
pub struct Bitstream {
    pub magic: u32,
    parser: StreamParser,
}

impl Bitstream {
    pub fn from_cursor(mut cursor: BitCursor) -> Result<Self> {
        if cursor.byte_len() % 4 != 0 {
            return Err(anyhow!("Bad Container: input is not 4-byte aligned."));
        }

        Ok(Self {
            magic: cursor.read(32)? as u32,
            parser: StreamParser::new(cursor),
        })
    }

    pub fn from(inner: Vec<u8>) -> Result<(Option<Signature>, Self)> {
        debug("Beginning inteligent parse.");

        let mut cursor = BitCursor::new(inner.clone());

        let magic = cursor.read(32)? as u32;

        if magic == BITCODE_WRAPPER_MAGIC {
            debug("Input looks like a bitcode wrapper!");

            let signature = Signature {
                magic,
                version: cursor.read(32)? as u32,
                offset: cursor.read(32)? as u32,
                size: cursor.read(32)? as u32,
                cpu_type: cursor.read(32)? as u32,
            };

            let actual_length = (signature.size as usize) + 20;
            let mut cur = BitCursor::new_with_len(inner, actual_length)?;

            cur.seek(SeekFrom::Start(signature.offset.into()))?;
            return Ok((Some(signature), Self::from_cursor(cur)?));
        }

        debug("Input is probably a raw bitstream!");
        Ok((None, Self::from_raw(inner)?))
    }

    pub fn from_raw(inner: Vec<u8>) -> Result<Self> {
        let cursor = BitCursor::new(inner);
        Self::from_cursor(cursor)
    }

    pub fn advance(&mut self) -> Result<StreamEntry> {
        self.parser.advance()
    }
}

impl Iterator for Bitstream {
    type Item = Result<StreamEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.advance() {
            Ok(entry) => {
                if matches!(entry, StreamEntry::EndOfStream) {
                    return None;
                }

                Some(Ok(entry))
            }
            Err(e) => Some(Err(e)),
        }
    }
}
