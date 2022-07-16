use super::{Access, Error, Identity};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// An extremely simple in-memory authorization database
pub struct AuthorizationDb {
    path: PathBuf,
    identities: HashMap<Identity, Access>,
}

impl AuthorizationDb {
    pub fn lookup(&self, identity: &Identity) -> Option<Access> {
        self.identities.get(identity).copied()
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref().to_path_buf();
        let identities = Self::read_file(&path)?;
        Ok(Self { path, identities })
    }

    pub fn reload<P: AsRef<Path>>(&mut self) -> Result<(), Error> {
        self.identities = Self::read_file(&self.path)?;
        Ok(())
    }

    fn read_file<P: AsRef<Path>>(path: P) -> Result<HashMap<Identity, Access>, Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut identities = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            let fields = line.split(',').collect::<Vec<_>>();

            if fields.len() != 3 {
                return Err(Error::InvalidAuthorizationDb);
            }

            let access = fields[0].parse::<Access>()?;
            let identity = Identity::from_pair(fields[1], fields[2])?;

            identities.insert(identity, access);
        }

        Ok(identities)
    }
}
