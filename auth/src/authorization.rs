use super::{Access, Identity, Provider};
use flagset::FlagSet;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// An extremely simple in-memory authorization database
pub struct Authorizations {
    identities: HashMap<Identity, FlagSet<Access>>,
}

impl Authorizations {
    pub fn lookup(&self, identity: &Identity) -> FlagSet<Access> {
        self.identities.get(identity).cloned().unwrap_or_default()
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let identities = Self::read_file(&path)?;
        Ok(Self { identities })
    }

    fn read_file<P: AsRef<Path>>(path: P) -> Result<HashMap<Identity, FlagSet<Access>>, Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut identities = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            let fields = line.split(',').collect::<Vec<_>>();

            if fields.len() != 4 {
                return Err(Error::InvalidLine(line));
            }

            let access = Access::from_label(fields[0])?;
            let provider = fields[1].parse::<Provider>()?;
            let identity = Identity::for_provider(provider, fields[2], fields[3])?;

            identities.insert(identity, access);
        }

        Ok(identities)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("Invalid Access")]
    InvalidAccess(String),
    #[error("Invalid provider")]
    InvalidProvider(String),
    #[error("Invalid identifier")]
    InvalidIdentifier(String),
    #[error("Invalid line")]
    InvalidLine(String),
}
