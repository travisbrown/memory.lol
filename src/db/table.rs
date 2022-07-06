use super::Error;
use rocksdb::DB;
use std::path::Path;

pub trait Table: Sized {
    type Counts;

    fn underlying(&self) -> &DB;
    fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error>;
    fn get_counts(&self) -> Result<Self::Counts, Error>;

    fn get_estimated_key_count(&self) -> Result<Option<u64>, Error> {
        Ok(self
            .underlying()
            .property_int_value("rocksdb.estimate-num-keys")?)
    }
}
