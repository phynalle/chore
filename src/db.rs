use error::Result;
use rocksdb::{Options, DB};

pub fn open_database() -> Result<DB> {
    let mut db_opts = Options::default();
    db_opts.create_if_missing(true);

    DB::open(&db_opts, "/Users/phynalle/.tmp/testdb").map_err(|e| e.into())
}
