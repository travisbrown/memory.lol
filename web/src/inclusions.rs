use super::error::Error;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Inclusions {
    ids: HashSet<u64>,
}

impl Inclusions {
    pub fn read_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let ids = reader
            .lines()
            .map(|result| {
                let line = result?;
                let id = line
                    .parse::<u64>()
                    .map_err(|_| Error::InvalidInclusionFileLine(line.to_string()))?;

                Ok(id)
            })
            .collect::<Result<HashSet<_>, Error>>()?;

        Ok(Self { ids })
    }

    pub fn contains(&self, id: u64) -> bool {
        self.ids.contains(&id)
    }
}
