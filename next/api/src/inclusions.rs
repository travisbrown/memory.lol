use super::error::Error;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// A set of user IDs that always receive full (unlimited) results.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Inclusions {
    ids: HashSet<u64>,
}

impl Inclusions {
    /// Read a set of user IDs from a file with one decimal ID per line.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the file cannot be read, or
    /// [`Error::InvalidInclusionFileLine`] if a line is not a valid ID.
    pub fn read_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let ids = reader
            .lines()
            .map(|result| {
                let line = result?;
                line.parse::<u64>()
                    .map_err(|_| Error::InvalidInclusionFileLine(line))
            })
            .collect::<Result<HashSet<_>, Error>>()?;

        Ok(Self { ids })
    }

    /// Check whether the given user ID is included.
    pub fn contains(&self, id: u64) -> bool {
        self.ids.contains(&id)
    }
}
