use anyhow::{Result, anyhow};
use std::io;

use super::debug;

/// A no-copy cursor wrapper for a bitstream.
///
/// Any type that implements `AsRef<[u8]>` can be used with `BitCursor`.
#[derive(Debug)]
pub struct BitCursor {
    /// The cursor-accessible length of the buffer. This is normally the same
    /// as the buffer's length, but can be shorter for uses where `inner`
    /// is a multi-purpose buffer.
    byte_len: usize,

    /// Our inner buffer.
    inner: Vec<u8>,

    /// Our current byte index in `inner`, which may be ahead of our
    /// current bit position (if `current_block` is not exhausted).
    byte_pos: usize,

    /// The last `u64`-sized block read from `inner`.
    current_block: u64,

    /// The number of bits in `current_block` that are valid (i.e., not
    /// yet consumed).
    bit_index: usize,
}

impl BitCursor {
    const BLOCK_SIZE: usize = std::mem::size_of::<u64>();
    const BLOCK_SIZE_BITS: usize = u64::BITS as usize;
    const MAX_VBR_BITS: usize = 32;

    /// Create a new `BitCursor` for the `inner` buffer.
    pub fn new(inner: Vec<u8>) -> Self {
        Self {
            byte_len: inner.len(),
            inner: inner,
            byte_pos: 0,
            current_block: 0,
            bit_index: 0,
        }
    }

    /// Create a new `BitCursor` for the `inner` buffer, limiting to `byte_len` bytes.
    ///
    /// Returns an error if `byte_len` exceeds `inner`'s range.
    pub fn new_with_len(inner: Vec<u8>, byte_len: usize) -> Result<Self> {
        if byte_len > inner.len() {
            return Err(anyhow!("Invalid length."));
        }

        Ok(Self {
            byte_len: byte_len,
            inner: inner,
            byte_pos: 0,
            current_block: 0,
            bit_index: 0,
        })
    }

    /// Return the length of the data wrapped by this cursor, in bytes.
    pub fn byte_len(&self) -> usize {
        self.byte_len
    }

    /// Return the length of the data wrapped by this cursor, in bits.
    pub fn bit_len(&self) -> usize {
        self.byte_len() * 8
    }

    /// Return the current position in the data, at bit granularity.
    pub fn tell_bit(&self) -> usize {
        (self.byte_pos * 8) - self.bit_index
    }

    /// Return the current position in the data, at byte granularity.
    pub fn tell_byte(&self) -> usize {
        self.tell_bit() / 8
    }

    /// Return whether the underlying data is "exhausted", i.e. whether it's
    /// impossible to read any further from the cursor's current position.
    pub fn exhausted(&self) -> bool {
        self.bit_index == 0 && self.byte_len() <= self.byte_pos
    }

    /// Seek to the given bit-granular position in the bitstream.
    ///
    /// NOTE: This is a bit-granular absolute seek. If you only need byte granularity
    /// or would like to do a relative (start or end) seek, use the [`Seek`](std::io::Seek)
    /// implementation.
    pub fn seek_bit(&mut self, pos: usize) -> Result<()> {
        println!();
        debug(&format!("seek_bit: seeking to {}", pos));

        // Get the byte corresponding to this bit.
        let byte_pos = (pos / 8) & !(Self::BLOCK_SIZE - 1);

        if byte_pos > self.byte_len() {
            return Err(anyhow!("End of file!"));
        }

        // Change our position, and clear any internal block state.
        self.byte_pos = byte_pos;
        self.clear_block_state();

        // Finally, we need to bring our internal block state into sync
        // with our bit position by consuming any bits at the current
        // word before our new position.
        // NOTE(ww): LLVM's BitstreamReader prefers the equivalent of
        // `pos & (usize::BITS - 1)`, presumably to avoid a modulo operation.
        // But (experimentally) LLVM is more than smart enough to optimize
        // this down to a single AND, so I used the modulo version here for
        // clarity.
        let bits_to_consume = pos % Self::BLOCK_SIZE_BITS;
        debug(&format!("bits_to_consume={}", bits_to_consume));
        if bits_to_consume > 0 {
            self.read(bits_to_consume)?;
        }

        Ok(())
    }

    /// Clear our internal block state.
    ///
    /// This should be called as part of any operation that modifies the cursor's
    /// position within the bitstream, as any change in position invalidates the
    /// block.
    fn clear_block_state(&mut self) {
        self.current_block = 0;
        self.bit_index = 0;
    }

    /// Fill the internal block state, updating our cursor position in the process.
    ///
    /// This tries to read up to `usize` bytes from the underlying data,
    /// reading fewer if a full block isn't available.
    fn load_current_block(&mut self) -> Result<()> {
        if self.tell_byte() >= self.byte_len() {
            return Err(anyhow!("End of file!"));
        }

        // NOTE(ww): We've consumed all of the bits in our current block, so clear our state.
        // This is essential to the correct behavior of `load_current_block`,
        // as it uses `tell_byte` to determine which byte to begin at for the next block load.
        self.clear_block_state();

        // Do either a full or a short read, depending on how much data
        // we have left.
        let block_bytes = if self.tell_byte() + Self::BLOCK_SIZE < self.byte_len() {
            &self.inner[self.tell_byte()..(self.tell_byte() + Self::BLOCK_SIZE)]
        } else {
            &self.inner[self.tell_byte()..self.byte_len()]
        };

        self.current_block = 0;
        for (idx, byte) in block_bytes.iter().enumerate() {
            self.current_block |= (*byte as u64) << (idx * 8);
        }

        // We've advanced by this many bytes.
        self.byte_pos += block_bytes.len();

        // We have this many valid bits in the current block.
        self.bit_index = block_bytes.len() * 8;

        debug(&format!(
            "load_current_block finished: current_block={}, bit_index={}",
            self.current_block, self.bit_index
        ));

        Ok(())
    }

    /// Read `nbits` bits of data at the current position. The data is returned
    /// as a `u64`.
    ///
    /// Returns an error if the requested read is invalid (e.g. EOF or not enough data)
    /// or if `nbits` is invalid (zero, or >= `u64::BITS`).
    pub fn read(&mut self, nbits: usize) -> Result<u64> {
        debug(&format!(
            "read: nbits={}, current_block={}, bit_index={}",
            nbits, self.current_block, self.bit_index
        ));

        if nbits == 0 || nbits >= Self::BLOCK_SIZE_BITS {
            return Err(anyhow!("Invalid Read Size."));
        }

        // If we have enough bits in the current block, steal them and
        // return fast.
        if self.bit_index >= nbits {
            debug(&format!("we have enough bits!"));

            let read = self.current_block & (!0 >> (Self::BLOCK_SIZE_BITS - nbits));

            self.current_block >>= nbits;
            self.bit_index -= nbits;

            return Ok(read);
        }

        // If we don't have enough bits, use the ones we have and fetch
        // a new `current_block`, completing the read with its contents.
        let bits_left = nbits - self.bit_index;
        let part_1 = if self.bit_index > 0 {
            self.current_block
        } else {
            0
        };

        self.load_current_block()?;

        // `load_current_block` might succeed, but might not load in enough
        // bits to fully service the read.
        if bits_left > self.bit_index {
            return Err(anyhow!("Short."));
        }

        let part_2 = self.current_block & (!0 >> (Self::BLOCK_SIZE_BITS - bits_left));

        self.current_block >>= bits_left;
        self.bit_index -= bits_left;

        debug(&format!(
            "part_2 done: current_block={}, bit_index={}",
            self.current_block, self.bit_index
        ));

        // Mash the parts together.
        Ok(part_1 | (part_2 << (nbits - bits_left)))
    }

    /// Read a `width`-wide VBR-encoded integer.
    ///
    /// This function returns only unsigned integers. For signed integers,
    /// use `read_svbr`.
    pub fn read_vbr(&mut self, width: usize) -> Result<u64> {
        // Sanity check: widths under 2 can't be VBR encodings, and, like LLVM itself,
        // we simply don't support widths above 32.
        if !(2..=Self::MAX_VBR_BITS).contains(&width) {
            return Err(anyhow!("Invalid VBR Width."));
        }

        let block_mask = 1 << (width - 1);

        // Read each VBR block until we encounter a block that doesn't include the
        // continuation bit.
        let mut result: u64 = 0;
        let mut shift = 0;
        loop {
            // Read a block, add it to the result (with the potential continuation bit masked off)
            let block = self.read(width)?;
            debug(&format!(
                "block: {:#b}, masked: {:#b}",
                block,
                block & !block_mask
            ));
            result |= (block & !block_mask) << shift;

            // If we don't have a continuation bit, then we're done with the VBR.
            let continuation = (block & block_mask) != 0;
            if !continuation {
                break;
            };

            // Calculate the shift needed for the next block.
            shift += width - 1;
        }

        Ok(result)
    }

    /// Return a `width`-side signed VBR-encoded integer from `cursor`.
    ///
    /// This function returns only signed integers, assuming LLVM's signed VBR
    /// representation.
    pub fn read_svbr(&mut self, width: usize) -> Result<isize> {
        let mut result = self.read_vbr(width)?;

        // The lowest bit indicates the actual sign: high for negative and low for positive.
        let sgn = (result & 1) != 0;
        result >>= 1;

        if sgn {
            Ok(-(result as isize))
        } else {
            Ok(result as isize)
        }
    }

    /// Align the stream on the next 32-bit boundary.
    ///
    /// Any data consumed during alignment is discarded.
    pub fn align32(&mut self) {
        debug(&format!("aligning the cursor"));

        if self.bit_index >= 32 {
            self.current_block >>= self.bit_index - 32;
            self.bit_index = 32;
        } else {
            self.clear_block_state();
        }
    }
}

/// A `Seek` implementation for `BitCursor`.
///
/// Seeking past the end of a `BitCursor` is always invalid, and always returns
/// an error.
///
/// NOTE: This is a byte-granular implementation of `Seek`.
/// For bit-granular seeking, use [`seek_bit`](BitCursor::seek_bit).
impl io::Seek for BitCursor {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        // Note the ugly as-casting below: we ultimately turn `off` into a
        // `usize` to make it compatible with indexing (since we always have
        // a backing buffer), but we first have to round-trip it through i64
        // for relative seeks.
        let off = match pos {
            io::SeekFrom::Start(pos) => pos,
            io::SeekFrom::End(pos) => {
                if pos >= 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "cannot seek past end",
                    ));
                }

                // Seeking backwards from the end is perfectly fine.
                ((self.byte_len() as i64) + pos) as u64
            }
            io::SeekFrom::Current(pos) => ((self.tell_byte() as i64) + pos) as u64,
        } as usize;

        // Sanity check: we can't seek before or beyond the backing buffer.
        // We can, however, seek to the exact end of the backing buffer, to
        // indicate an EOF condition.
        // We don't need to check for a negative offset here, since we've cast
        // back into the land of unsigned integers.
        if off > self.byte_len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "impossible seek requested",
            ));
        }

        // Actually update our location.
        self.byte_pos = off;

        // Regardless of the kind of seek, we reset our current block state to ensure that any
        // subsequent reads are correct.
        self.clear_block_state();

        Ok(off as u64)
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(self.tell_byte() as u64)
    }

    // TODO(ww): Supply this when it's stabilized.
    // fn stream_len(&mut self) -> io::Result<u64> {
    //     Ok(self.byte_len() as u64)
    // }
}
