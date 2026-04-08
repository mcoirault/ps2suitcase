use chrono::NaiveDateTime;
use std::fmt::{Display, Formatter, Result};
use std::io::Cursor;

pub const DIR_ID: u16 = 0x8427;
pub const FILE_ID: u16 = 0x8497;

pub const PAGE_SIZE: u32 = 0x400;

#[derive(Default, Clone)]
pub struct PSU {
    pub entries: Vec<PSUEntry>,
}

impl PSU {
    pub fn add_defaults(&mut self, name: &str, file_count: usize, timestamp: NaiveDateTime) {
        self.entries.push(PSUEntry {
            id: DIR_ID,
            size: file_count as u32 + 2, // +2 to include . and ..
            created: timestamp,
            sector: 0,
            modified: timestamp,
            name: name.to_owned(),
            kind: PSUEntryKind::Directory,
            contents: None,
        });
        self.entries.push(PSUEntry {
            id: DIR_ID,
            size: 0,
            created: timestamp,
            sector: 0,
            modified: timestamp,
            name: ".".to_string(),
            kind: PSUEntryKind::Directory,
            contents: None,
        });
        self.entries.push(PSUEntry {
            id: DIR_ID,
            size: 0,
            created: timestamp,
            sector: 0,
            modified: timestamp,
            name: "..".to_string(),
            kind: PSUEntryKind::Directory,
            contents: None,
        });
    }
}

impl Display for PSU {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut output = format!(
            "{:12}| {:16}| {:9}| {:25}| {:25}",
            "Type", "Name", "Size", "Created", "Modified"
        );
        output = format!("{}\n{:-<99}", output, "");
        for entry in self.entries.clone() {
            output = format!(
                "{}\n{:12}| {:16}| {:9}| {:25}| {:25}",
                output,
                entry.kind.to_string(),
                entry.name,
                entry.size,
                entry.created.format("%Y-%m-%d %H:%M:%S"),
                entry.modified.format("%Y-%m-%d %H:%M:%S"),
            );
        }
        write!(f, "{}", output)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PSUEntryKind {
    Directory,
    File,
}

impl Display for PSUEntryKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            PSUEntryKind::Directory => write!(f, "directory"),
            PSUEntryKind::File => write!(f, "file"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PSUEntry {
    pub id: u16,
    pub size: u32,
    pub created: NaiveDateTime,
    pub sector: u16,
    pub modified: NaiveDateTime,
    pub name: String,
    pub kind: PSUEntryKind,
    pub contents: Option<Vec<u8>>,
}

pub(crate) struct PSUParser {
    pub(crate) c: Cursor<Vec<u8>>,
    pub(crate) len: u64,
}
