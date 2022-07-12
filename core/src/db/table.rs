use super::Error;
use rocksdb::DB;

pub trait Mode {
    fn is_read_only() -> bool;
}

pub struct ReadOnly;
pub struct Writeable;

impl Mode for ReadOnly {
    fn is_read_only() -> bool {
        true
    }
}
impl Mode for Writeable {
    fn is_read_only() -> bool {
        false
    }
}

pub trait Table: Sized {
    type Counts;

    fn underlying(&self) -> &DB;
    fn get_counts(&self) -> Result<Self::Counts, Error>;

    fn get_estimated_key_count(&self) -> Result<Option<u64>, Error> {
        Ok(self
            .underlying()
            .property_int_value("rocksdb.estimate-num-keys")?)
    }
}
