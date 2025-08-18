use std::collections::HashMap;

use anyhow::{Result, anyhow};

use super::{
    Abbrev, AbbrevId, BitCursor, Block, BlockInfoCode, FIRST_APPLICATION_ABBREV_ID, Fields, Record,
    ReservedAbbrevId, ReservedBlockId, debug,
};

pub const INITIAL_ABBREV_ID_WIDTH: u64 = 2;

#[derive(Debug)]
pub struct StreamParser {
    cursor: BitCursor,
    scopes: Vec<Scope>,
    blockinfo: HashMap<u64, Vec<Abbrev>>,
}

impl StreamParser {
    pub fn new(cursor: BitCursor) -> Self {
        Self {
            cursor,
            scopes: vec![Scope::default()],
            blockinfo: Default::default(),
        }
    }

    fn scope(&self) -> &Scope {
        #[allow(clippy::unwrap_used)]
        self.scopes.last().unwrap()
    }

    fn scope_mut(&mut self) -> &mut Scope {
        #[allow(clippy::unwrap_used)]
        self.scopes.last_mut().unwrap()
    }

    pub fn advance(&mut self) -> Result<StreamEntry> {
        if self.cursor.exhausted() {
            return Ok(StreamEntry::EndOfStream);
        }

        debug(&format!(
            "advancing, current scope {:?} @ bit position {}",
            self.scope(),
            self.cursor.tell_bit()
        ));

        let id: AbbrevId = self
            .cursor
            .read(self.scope().abbrev_id_width() as usize)?
            .into();

        debug(&format!("Next entry ID: {:?}", id));

        match id {
            AbbrevId::Reserved(res) => match res {
                ReservedAbbrevId::END_BLOCK => Ok(match self.exit_block()? {
                    Some(exit) => exit,
                    None => self.advance()?,
                }),
                ReservedAbbrevId::ENTER_SUBBLOCK => Ok(match self.enter_block()? {
                    Some(enter) => enter,
                    None => self.advance()?,
                }),
                ReservedAbbrevId::DEFINE_ABBREV => {
                    self.define_abbrev()?;
                    self.advance()
                }
                ReservedAbbrevId::UNABBREV_RECORD => Ok(match self.parse_unabbrev()? {
                    Some(unabbrev) => unabbrev,
                    None => self.advance()?,
                }),
            },
            AbbrevId::Defined(abbrev_id) => Ok(match self.parse_with_abbrev(abbrev_id)? {
                Some(defined) => defined,
                None => self.advance()?,
            }),
        }
    }

    pub fn parse_with_abbrev(&mut self, abbrev_id: u64) -> Result<Option<StreamEntry>> {
        let abbrev = self.scope().get_abbrev(abbrev_id)?.clone();

        let mut fields = abbrev.parse(&mut self.cursor)?;
        debug(&format!("Parsed fields: {:?}", fields));

        let code = fields.remove(0);

        if self.scope().is_blockinfo() {
            return Ok(None);
        }

        Ok(Some(StreamEntry::Record(Record {
            abbrev_id: Some(abbrev_id),
            code,
            fields,
        })))
    }

    pub fn parse_unabbrev(&mut self) -> Result<Option<StreamEntry>> {
        if matches!(self.scope(), Scope::Initial) {
            return Err(anyhow!("UNABBREV_RECORD outside of any block scope."));
        }

        let code = self.cursor.read_vbr(6)?;
        let num_operands = self.cursor.read_vbr(6)?;

        debug(&format!(
            "UnabbrevRecord code={}, num_operands={}",
            code, num_operands
        ));

        let mut fields: Fields = Vec::with_capacity(num_operands as usize);

        for _ in 0..num_operands {
            fields.push(self.cursor.read_vbr(6)?);
        }

        let record = Record::from_unabbrev(code, fields);
        if self.scope().is_blockinfo() {
            let code = BlockInfoCode::from_u64(record.code)?;
            match code {
                BlockInfoCode::SETBID => {
                    let block_id = record.fields[0];
                    debug(&format!("SETBID: BLOCKINFO block ID is now {}", block_id));
                    self.scope_mut().set_blockinfo_block_id(block_id)?;
                }
                BlockInfoCode::BLOCKNAME => debug("Skipping BLOCKNAME code in BLOCKINFO..."),
                BlockInfoCode::SETRECORDNAME => {
                    debug(&format!("Skipping SETRECORDNAME code in BLOCKINFO..."))
                }
            }
        }

        Ok(Some(StreamEntry::Record(record)))
    }

    pub fn define_abbrev(&mut self) -> Result<()> {
        let abbrev = Abbrev::new(&mut self.cursor)?;
        debug(&format!("New Abbrev: {:?}", abbrev));

        if self.scope().is_blockinfo() {
            let block_id = self.scope().blockinfo_block_id().ok_or(anyhow!(
                "DEFINE_ABBREV in BLOCKINFO, but no preceding SETBID."
            ))?;

            self.blockinfo
                .entry(block_id)
                .or_insert_with(Vec::new)
                .push(abbrev);
        } else {
            self.scope_mut().extend_abbrevs(std::iter::once(abbrev))?;
        }

        Ok(())
    }

    pub fn enter_block(&mut self) -> Result<Option<StreamEntry>> {
        let block_id = self.cursor.read_vbr(8)?;
        let new_width = self.cursor.read_vbr(4)?;

        self.cursor.align32();

        if new_width < 1 {
            return Err(anyhow!(
                "can't enter block: invalid code size '{:?}'",
                new_width
            ));
        }

        let block_len = self.cursor.read(32)? * 4;

        debug(&format!(
            "entered block: ID={}, new_abbrev_width={}, block_len={} @ bit position {}",
            block_id,
            new_width,
            block_len,
            self.cursor.tell_bit()
        ));

        self.scopes.push(Scope::new(new_width, block_id));

        if let Some(abbrevs) = self.blockinfo.get(&block_id).map(|a| a.to_vec()) {
            self.scope_mut().extend_abbrevs(abbrevs)?;
        }

        if self.scope().is_blockinfo() {
            return Ok(None);
        }

        Ok(Some(StreamEntry::SubBlock(Block {
            block_id,
            len: block_len,
        })))
    }

    pub fn exit_block(&mut self) -> Result<Option<StreamEntry>> {
        self.cursor.align32();

        if self.scopes.len() <= 1 {
            return Err(anyhow!(
                "Malformed stream: Cannot perform END_BLOCK because scope stack is empty."
            ));
        }

        #[allow(clippy::unwrap_used)]
        let scope = self.scopes.pop().unwrap();

        debug(&format!(
            "exit_block: new active scope is {:?}",
            self.scope()
        ));

        if scope.is_blockinfo() {
            return Ok(None);
        }

        Ok(Some(StreamEntry::EndBlock))
    }
}

#[derive(Debug, Default)]
pub enum Scope {
    #[default]
    Initial,
    Block {
        abbrev_id_width: u64,
        block_id: u64,
        blockinfo_block_id: Option<u64>,
        abbrevs: Vec<Abbrev>,
    },
}

impl Scope {
    pub fn new(abbrev_id_width: u64, block_id: u64) -> Self {
        Self::Block {
            abbrev_id_width,
            block_id,
            blockinfo_block_id: None,
            abbrevs: vec![],
        }
    }

    pub fn abbrev_id_width(&self) -> u64 {
        match self {
            Scope::Initial => INITIAL_ABBREV_ID_WIDTH,
            Scope::Block {
                abbrev_id_width, ..
            } => *abbrev_id_width,
        }
    }

    pub fn is_blockinfo(&self) -> bool {
        match self {
            Scope::Initial => false,
            Scope::Block { block_id, .. } => *block_id == ReservedBlockId::BLOCKINFO as u64,
        }
    }

    pub fn blockinfo_block_id(&self) -> Option<u64> {
        match self {
            Scope::Initial => None,
            Scope::Block {
                blockinfo_block_id, ..
            } => *blockinfo_block_id,
        }
    }

    pub fn set_blockinfo_block_id(&mut self, new_block_id: u64) -> Result<()> {
        if let Scope::Block {
            blockinfo_block_id, ..
        } = self
        {
            *blockinfo_block_id = Some(new_block_id);
            return Ok(());
        }

        Err(anyhow!(
            "Can't set a BLOCKINFO_BLOCK_ID to a Non-BLOCKINFO scope."
        ))
    }

    pub fn get_abbrev(&self, abbrev_id: u64) -> Result<&Abbrev> {
        match self {
            Scope::Initial => Err(anyhow!("Non-block scope cannot contain records.")),
            Scope::Block { abbrevs, .. } => {
                let idx = abbrev_id - FIRST_APPLICATION_ABBREV_ID;
                abbrevs
                    .get(idx as usize)
                    .ok_or(anyhow!("Bad Abbrev: {}", abbrev_id))
            }
        }
    }

    pub fn extend_abbrevs(
        &mut self,
        new_abbrevs: impl std::iter::IntoIterator<Item = Abbrev>,
    ) -> Result<()> {
        match self {
            Scope::Initial => Err(anyhow!("Non-block scope cannot reference abbreviations.")),
            Scope::Block { abbrevs, .. } => {
                abbrevs.extend(new_abbrevs);
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub enum StreamEntry {
    EndBlock,
    SubBlock(Block),
    Record(Record),
    EndOfStream,
}
