use crate::common::Idx;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub struct ILine<'a> {
    pub ptr: &'a str,
    pub size: usize,
}

pub struct InputFile {
    pub lines: Vec<String>,
}

impl InputFile {
    pub fn from_file(filename: &str) -> io::Result<Self> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        let lines = reader.lines()
            .map(|l| l.unwrap_or_default())
            .collect::<Vec<_>>();

        Ok(InputFile { lines })
    }

    pub fn ifetch(&self, line: Idx) -> Option<ILine<'_>> {
        self.lines.get(line - 1).map(|s| ILine { ptr: s, size: s.len() })
    }

    pub fn num_lines(&self) -> Idx {
        self.lines.len()
    }
}