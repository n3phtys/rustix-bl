use config::StaticConfig;
use lmdb;
use datastore::Datastore;
use rustix_event_shop::Event;
use rustix_event_shop::BLEvents;
use serde_json::Error as Error_JSON;
use lmdb::Error as Error_LMDB;
use lmdb::EnvironmentBuilder;
use lmdb::Environment;
use lmdb::Database;
use lmdb::DatabaseFlags;
use lmdb::RwTransaction;
use lmdb::RoTransaction;
use lmdb::RoCursor;
use lmdb::WriteFlags;
use std::str;
use std::path::Path;
use lmdb::Cursor;
use lmdb::Transaction;
use std::marker::Sized;
use std::convert::AsRef;
use bincode::{deserialize, serialize, Infinite};
use serde_json;
use std;
use std::error::Error;
use std::fmt;
use errors;

quick_error! {
    #[derive(Debug)]
    pub enum RustixError {
        /// DB Error
        DB(err: Error_LMDB) {}
        /// Serialization Error
        SerialJson(err: Error_JSON) {}
        /// Utf8 Error
        SerialUTF8(err: std::str::Utf8Error) {}
        /// My own Error
        Init(err: errors::custom_errors::CustomRustixFrontendError) {}
        ///other Error
        Other(err: Box<std::error::Error>) {
            cause(&**err)
            description(err.description())
        }
    }
}


impl std::convert::From<errors::custom_errors::CustomRustixFrontendError> for RustixError {
    fn from(e: errors::custom_errors::CustomRustixFrontendError) -> Self {
        return RustixError::Init(e);
    }
}

impl std::convert::From<Error_JSON> for RustixError {
    fn from(e: Error_JSON) -> Self {
        return RustixError::SerialJson(e);
    }
}

impl std::convert::From<std::str::Utf8Error> for RustixError {
    fn from(e: std::str::Utf8Error) -> Self {
        return RustixError::SerialUTF8(e);
    }
}

impl std::convert::From<Error_LMDB> for RustixError {
    fn from(e: Error_LMDB) -> Self {
        return RustixError::DB(e);
    }
}


pub trait Persistencer {
    fn test_store_apply(&mut self, event: &BLEvents, datastore: &mut Datastore) -> bool;

    //returns number of events loaded
    fn reload_from_filepath(&mut self, datastore: &mut Datastore) -> Result<u64, RustixError>;
    //fn initialize(&mut self, datastore: &mut Datastore) -> Result<u32, RustixError>;
}

#[derive(Debug)]
pub struct LmdbDb {
    pub db: lmdb::Database,
    pub db_env: lmdb::Environment,
}

#[derive(Debug)]
pub struct FilePersister {
    pub config: StaticConfig,
    pub lmdb: Option<LmdbDb>,
    pub events_stored: u64,
}

impl FilePersister {
    pub fn new(config: StaticConfig) -> Result<Self, lmdb::Error> {
        let lmdb = if config.use_persistence {
            let dir: &std::path::Path = std::path::Path::new(&config.persistence_file_path);
            let db_flags: lmdb::DatabaseFlags = lmdb::DatabaseFlags::empty();
            println!("trying to get env");
            let db_environment = try!(lmdb::Environment::new().set_max_dbs(1).open(&dir));
            println!("trying to get database");
            let database = try!(db_environment.create_db(None, db_flags));
            println!("gotten database");
            Some(LmdbDb {
                db: database,
                db_env: db_environment,
            })
        } else {
            None
        };
        println!("first part finished");

        let mut fp = FilePersister {
            config: config,
            lmdb: lmdb,
            events_stored: 0,
        };

        return Ok(fp);
    }
}


pub trait LMDBPersistencer {
    fn store_event_in_db(&mut self, id: u64, event: &BLEvents) -> Result<(), RustixError>;
    fn increment_counter(&mut self) -> ();
    fn get_counter(&self) -> u64;
}

fn transform_u32_to_array_of_u8(x: u32) -> [u8; 4] {
    let b1: u8 = ((x >> 24) & 0xff) as u8;
    let b2: u8 = ((x >> 16) & 0xff) as u8;
    let b3: u8 = ((x >> 8) & 0xff) as u8;
    let b4: u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4];
}

impl LMDBPersistencer for FilePersister {
    fn store_event_in_db(&mut self, id: u64, event: &BLEvents) -> Result<(), RustixError> {
        match self.lmdb {
            Some(ref lmdb) => {
                let mut rw_transaction: RwTransaction = try!(lmdb.db_env.begin_rw_txn());
                let tx_flags: WriteFlags = WriteFlags::empty();
                let key = id.to_string().into_bytes();//   transform_u32_to_array_of_u8(id);
                let data = try!(serde_json::to_string(event));
                let result = rw_transaction.put(lmdb.db, &key, &data, tx_flags);
                try!(rw_transaction.commit());
            }
            None => (),
        }
        return Ok(self.increment_counter());
    }


    fn increment_counter(&mut self) -> () {
        self.events_stored = self.events_stored + 1;
    }
    fn get_counter(&self) -> u64 {
        return self.events_stored;
    }
}

impl Persistencer for FilePersister {
    fn test_store_apply(&mut self, event: &BLEvents, datastore: &mut Datastore) -> bool {
        let allowed = event.can_be_applied(datastore);
        if allowed {
            let id: u64 = self.get_counter() + 1u64;
            match self.store_event_in_db(id, event) {
                Err(e) => return false,
                Ok(t) => {
                    return event.apply(datastore, &self.config);
                }
            }
        } else {
            return false;
        }
    }

    fn reload_from_filepath(&mut self, datastore: &mut Datastore) -> Result<u64, RustixError> {
        let mut counter = self.get_counter();
        //TODO: only check starting from store event counter (to deal with snapshots)

        println!("Reloading events from lmdb with counter = {}", counter);

        match self.lmdb {
            Some(ref lmdb) => {
                let tx = try!(lmdb.db_env.begin_ro_txn());
                {
                    let mut cursor: RoCursor = try!(tx.open_ro_cursor(lmdb.db));
                    let key = counter.to_string().into_bytes();
                    let iter = if counter != 0u64 {
                        cursor.iter_from(key)
                    } else {
                        cursor.iter_start()
                    };
                    for keyvalue in iter {
                        let (key, value) = keyvalue;
                        let json = try!(str::from_utf8(value));
                        println!("{:?}", json);
                        let event: BLEvents = try!(serde_json::from_str(json));
                        if event.can_be_applied(datastore) {
                            event.apply(datastore, &self.config);
                            counter = counter + 1u64;
                        }
                    }
                }
            }
            None => (),
        }

        return Ok(counter);
    }
}
