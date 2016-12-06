use std::io::BufReader;
use std::io::prelude::*;
use std::fs::File;
use std::usize;

pub struct MemMapTable {
    entries: Vec<MapEntry>
}

impl MemMapTable {
    pub fn new() -> MemMapTable {
        let f = File::open("/proc/self/maps").expect("Couldn't open /proc/self/maps");
        let reader = BufReader::new(f);

        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line.expect("Couldn't read line from /proc/self/maps");
            println!("line = `{}`", line);

            let mut col_splitter = line.split(' ');

            let first_col = col_splitter.next().expect("Couldn't read first column");
            let mut splitter = first_col.splitn(2, '-');

            let start = splitter.next().unwrap();
            let end = splitter.next().unwrap();

            let perms = col_splitter.next().expect("Couldn't read perms");

            if perms.as_bytes()[2] != 'x' as u8 {
                // only interested in executable mappings
                continue;
            }

            let offset = col_splitter.next().expect("Couldn't read offset");

            let last_col = line.rsplit(' ').next().expect("Couldn't read last column");

            entries.push(MapEntry {
                start: usize::from_str_radix(start, 16).unwrap(),
                end: usize::from_str_radix(end, 16).unwrap(),
                offset: usize::from_str_radix(offset, 16).unwrap(),
                pathname: last_col.into(),
            });
        }

        MemMapTable {
            entries: entries
        }
    }

    pub fn find_entry(&self, pc: usize) -> Option<&MapEntry> {
        for entry in &self.entries {
            if entry.start <= pc && pc < entry.end {
                return Some(entry);
            }
        }

        None
    }
}

pub struct MapEntry {
    pub start: usize,
    pub end: usize,
    pub offset: usize,
    pub pathname: String,
}

impl MapEntry {
    pub fn address(&self, pc: usize) -> usize {
        assert!(self.start <= pc && pc < self.end);

        self.offset + pc - self.start
    }
}