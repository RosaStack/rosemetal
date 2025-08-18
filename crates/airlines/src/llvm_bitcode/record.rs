pub type Fields = Vec<u64>;

#[derive(Debug)]
pub struct Block {
    pub block_id: u64,
    pub len: u64,
}

#[derive(Clone, Debug)]
pub struct Record {
    pub abbrev_id: Option<u64>,
    pub code: u64,
    pub fields: Fields,
}

impl Record {
    pub fn from_unabbrev(code: u64, fields: Fields) -> Self {
        Self {
            abbrev_id: None,
            code,
            fields,
        }
    }
}
