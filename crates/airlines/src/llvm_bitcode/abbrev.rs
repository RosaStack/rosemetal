use anyhow::{Result, anyhow};

use super::{AbbrevOpEncoding, BitCursor, Fields, ReservedAbbrevId, debug};

pub const CHAR6_ALPHABET: &[u8] =
    b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._";

#[derive(Clone, Debug)]
pub struct Abbrev {
    pub operands: Vec<AbbrevOp>,
}

impl Abbrev {
    pub fn new(cursor: &mut BitCursor) -> Result<Self> {
        let num_abbrev_operands = cursor.read_vbr(5)?;

        if num_abbrev_operands < 1 {
            return Err(anyhow!("Expected at least one abbrev operand."));
        }

        debug(&format!("Expecting {} operands...", num_abbrev_operands));

        let mut operands: Vec<AbbrevOp> = vec![];
        let mut done_early = false;

        for idx in 0..num_abbrev_operands {
            let is_literal = cursor.read(1)? == 1;

            if is_literal {
                let value = cursor.read_vbr(8)?;

                operands.push(AbbrevOp::Literal(value));

                continue;
            }

            let encoding = AbbrevOpEncoding::from_u64(cursor.read(3)?)?;

            let operand = match encoding {
                AbbrevOpEncoding::Fixed => AbbrevOp::Fixed(cursor.read_vbr(5)?),
                AbbrevOpEncoding::Vbr => AbbrevOp::Vbr(cursor.read_vbr(5)?),
                AbbrevOpEncoding::Array => {
                    if idx != num_abbrev_operands - 2 {
                        return Err(anyhow!("Array Operand at invalid index."));
                    }

                    cursor.read(1)?;
                    let elem_encoding = AbbrevOpEncoding::from_u64(cursor.read(3)?)?;
                    done_early = true;

                    let elem = match elem_encoding {
                        AbbrevOpEncoding::Fixed => AbbrevOp::Fixed(cursor.read_vbr(5)?),
                        AbbrevOpEncoding::Vbr => AbbrevOp::Vbr(cursor.read_vbr(5)?),
                        AbbrevOpEncoding::Char6 => AbbrevOp::Char6,
                        _ => {
                            return Err(anyhow!(
                                "Blobs and Arrays cannot themselves be member types."
                            ));
                        }
                    };

                    AbbrevOp::Array(Box::new(elem))
                }
                AbbrevOpEncoding::Char6 => AbbrevOp::Char6,
                AbbrevOpEncoding::Blob => {
                    if idx != num_abbrev_operands - 1 {
                        return Err(anyhow!("Blob Operand at Invalid Index."));
                    }

                    AbbrevOp::Blob
                }
            };

            operands.push(operand);

            if done_early {
                break;
            }
        }

        Ok(Self { operands })
    }

    pub fn parse(&self, cursor: &mut BitCursor) -> Result<Fields> {
        Ok(self
            .operands
            .iter()
            .map(|operand| operand.parse(cursor))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AbbrevId {
    Reserved(ReservedAbbrevId),
    Defined(u64),
}

impl From<u64> for AbbrevId {
    fn from(value: u64) -> Self {
        match ReservedAbbrevId::from_u64(value) {
            Ok(result) => Self::Reserved(result),
            Err(_e) => Self::Defined(value),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AbbrevOp {
    Literal(u64),
    Vbr(u64),
    Fixed(u64),
    Array(Box<AbbrevOp>),
    Char6,
    Blob,
}

impl AbbrevOp {
    pub fn parse(&self, cursor: &mut BitCursor) -> Result<Fields> {
        Ok(match self {
            AbbrevOp::Literal(value) => vec![*value],
            AbbrevOp::Vbr(width) => vec![cursor.read_vbr(*width as usize)?],
            AbbrevOp::Fixed(width) => vec![cursor.read(*width as usize)?],
            AbbrevOp::Array(element) => {
                let array_len = cursor.read_vbr(6)? as usize;

                let mut fields: Fields = Vec::with_capacity(array_len);
                for _ in 0..array_len {
                    fields.extend(element.parse(cursor)?);
                }

                fields
            }
            AbbrevOp::Char6 => vec![Self::decode_char6(cursor.read(6)?) as u64],
            AbbrevOp::Blob => {
                let blob_len = cursor.read_vbr(6)? as usize;
                cursor.align32();

                let mut fields: Fields = Vec::with_capacity(blob_len);
                for _ in 0..blob_len {
                    fields.push(cursor.read(8)?);
                }

                cursor.align32();

                fields
            }
        })
    }

    pub fn decode_char6(char6: u64) -> u8 {
        CHAR6_ALPHABET[char6 as usize]
    }
}
