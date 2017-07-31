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
use bincode::{serialize, deserialize, Infinite};
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
    fn reload_from_filepath(&mut self, datastore: &mut Datastore) -> Result<u32, RustixError>; //returns number of events loaded
    //fn initialize(&mut self, datastore: &mut Datastore) -> Result<u32, RustixError>;
}

pub struct FilePersister {
    pub config: StaticConfig,
    pub db: lmdb::Database,
    pub db_env: lmdb::Environment,
    pub events_stored: u32,
}

pub struct TransientPersister{
    pub events_stored: u32
}

impl LMDBPersistencer for TransientPersister {
    fn store_event_in_db(&mut self, id: u32, event: &BLEvents) -> Result<(), RustixError> {
        return Ok(self.increment_counter());
    }


    fn increment_counter(&mut self) -> () {
        self.events_stored = self.events_stored + 1;
    }
    fn get_counter(&self) -> u32 {
        return self.events_stored;
    }
}

impl Persistencer for TransientPersister {
    fn test_store_apply(&mut self, event: &BLEvents, datastore: &mut Datastore) -> bool {
        let allowed = event.can_be_applied(datastore);
        if allowed {
            let id: u32 = self.get_counter() + 1u32;
            match self.store_event_in_db(id, event) {
                Err(e) => return false,
                Ok(t) => {
                    event.apply(datastore);
                    return true;
                }
            }
        } else {
            return false;
        }
    }

    fn reload_from_filepath(&mut self, datastore: &mut Datastore) -> Result<u32, RustixError> {
        let counter = 0u32;
        return Ok(counter);
    }
}

pub trait LMDBPersistencer {
    fn store_event_in_db(&mut self, id: u32, event: &BLEvents) -> Result<(), RustixError>;
    fn increment_counter(&mut self) -> ();
    fn get_counter(&self) -> u32;
}

fn transform_u32_to_array_of_u8(x: u32) -> [u8; 4] {
    let b1: u8 = ((x >> 24) & 0xff) as u8;
    let b2: u8 = ((x >> 16) & 0xff) as u8;
    let b3: u8 = ((x >> 8) & 0xff) as u8;
    let b4: u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4];
}

impl LMDBPersistencer for FilePersister {
    fn store_event_in_db(&mut self, id: u32, event: &BLEvents) -> Result<(), RustixError> {
        {
            let mut rw_transaction: RwTransaction = try!(RwTransaction::new(&self.db_env));
            let tx_flags: WriteFlags = WriteFlags::empty();
            let key = transform_u32_to_array_of_u8(id);
            let data = try!(serde_json::to_string(event));
            let result = rw_transaction.put(self.db, &key, &data, tx_flags);
            try!(rw_transaction.commit());
        }
        return Ok(self.increment_counter());
    }


    fn increment_counter(&mut self) -> () {
        self.events_stored = self.events_stored + 1;
    }
    fn get_counter(&self) -> u32 {
        return self.events_stored;
    }
}

impl Persistencer for FilePersister {
    fn test_store_apply(&mut self, event: &BLEvents, datastore: &mut Datastore) -> bool {
        let allowed = event.can_be_applied(datastore);
        if allowed {
            let id: u32 = self.get_counter() + 1u32;
            match self.store_event_in_db(id, event) {
                Err(e) => return false,
                Ok(t) => {
                    event.apply(datastore);
                    return true;
                }
            }
        } else {
            return false;
        }
    }

    fn reload_from_filepath(&mut self, datastore: &mut Datastore) -> Result<u32, RustixError> {
        let mut counter = 0u32;
        let env = &self.db_env;
        let db = self.db;
        let raw_ro_transaction = try!(RoTransaction::new(&env));
        {
            let mut read_transaction: RoCursor = try!(RoCursor::new(&raw_ro_transaction, db));

            for keyvalue in read_transaction.iter_start() {
                let (key, value) = keyvalue;
                let json = try!(str::from_utf8(value));
                println!("{:?}", json);
                let event: BLEvents = try!(serde_json::from_str(json));
                if event.can_be_applied(datastore) {
                    event.apply(datastore);
                    counter = counter + 1u32;
                }
            }
        }
        try!(raw_ro_transaction.commit());


        return Ok(counter);
    }
}