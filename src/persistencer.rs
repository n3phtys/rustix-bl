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

quick_error! {
    #[derive(Debug)]
    pub enum RustixError {
        /// DB Error
        DB(err: Error_LMDB) {}
        /// Serialization Error
        SerialJson(err: Error_JSON) {}
        /// Utf8 Error
        SerialUTF8(err: std::str::Utf8Error) {}
        ///other Error
        Other(err: Box<std::error::Error>) {
            cause(&**err)
            description(err.description())
        }
    }
}




#[derive(Debug)]
pub struct CustomRustixError {
    err: String,
}

impl Error for CustomRustixError {
    fn description(&self) -> &str {
        "Something bad happened"
    }
}

impl fmt::Display for CustomRustixError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Oh no, something bad went down")
    }
}


pub trait Persistencer {
    fn test_store_apply(&mut self, event: &BLEvents, datastore: &mut Datastore) -> bool;
    fn reload_from_filepath(&mut self, datastore: &mut Datastore) -> Result<u32, RustixError>; //returns number of events loaded
    fn initialize(&mut self, datastore: &mut Datastore) -> Result<u32, RustixError>;
}

pub struct FilePersister {
    pub config: StaticConfig,
    db: Option<lmdb::Database>,
    db_env: Option<lmdb::Environment>,
    events_stored: u32,
}

pub trait LMDBPersistencer {
    fn store_event_in_db(&mut self, event: &BLEvents) -> Result<(), RustixError>;
    fn get_env(&self) -> Result<&lmdb::Environment, RustixError>;
    fn get_db(&self) -> Result<lmdb::Database, RustixError>;
}

impl LMDBPersistencer for FilePersister {
    fn store_event_in_db(&mut self, event: &BLEvents) -> Result<(), RustixError> {
        let mut rw_transaction: RwTransaction = try!(RwTransaction::new(&try!(self.db_env.ok_or(CustomRustixError{err:"LMDB Environment not initialized".to_string()}))));
        let tx_flags: WriteFlags = WriteFlags::empty();
        let key = unimplemented!();
        let data = try!(serde_json::to_string(event));
        let result = rw_transaction.put(self.get_db()?, &key, &data, tx_flags );
        try!(rw_transaction.commit());
    }


    fn get_env(&self) -> Result<&lmdb::Environment, RustixError> {
        let environment: lmdb::Environment = try!(self.db_env.ok_or(CustomRustixError{err:"LMDB Environment not initialized".to_string()}));
        return Ok(&environment);
    }

    fn get_db(&self) -> Result<lmdb::Database, RustixError> {
       return Ok(try!(self.db.ok_or(CustomRustixError{err: "LMDB Database not initialized".to_string()})));
    }
}

impl Persistencer for FilePersister {
    fn test_store_apply(&mut self, event: &BLEvents, datastore: &mut Datastore) -> bool {
        let allowed = event.can_be_applied(datastore);
        if allowed {
            self.store_event_in_db(event);
            event.apply(datastore);
            return true;
        } else {
            return false;
        }
    }

    fn reload_from_filepath(&mut self, datastore: &mut Datastore) -> Result<u32, RustixError> {
        let mut counter = 0u32;
        match self.db_env {
            None => return Err(CustomRustixError{err: "LMDB Environment not initialized".to_string()}),
            Some(env) => {
                match self.db {
                    None => return Err(CustomRustixError{err: "LMDB Database not initialized".to_string()}),
                    Some(db) => {
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
                    }
                }
            }
        }
        return Ok(counter);
    }

    fn initialize(&mut self, datastore: &mut Datastore) -> Result<u32, RustixError> {
        self.db_env = Some(try!(Environment::new().open(self.config.database_filepath.as_ref())));
        self.db = Some(
            try!(
                try!(
                    self.get_env())
                    .create_db(Some("rustix_events")
                               , DatabaseFlags::empty())));
        let counter = try!(self.reload_from_filepath(datastore));
        self.events_stored = counter;
        return Ok(counter);
    }
}