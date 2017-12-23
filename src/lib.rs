// An attribute to hide warnings for unused code.
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#[macro_use]
pub extern crate derive_builder;
pub extern crate lmdb;
pub extern crate serde;
pub extern crate serde_json;


pub extern crate suffix_rs;

pub extern crate bincode;
#[macro_use]
pub extern crate serde_derive;

#[macro_use]
pub extern crate quick_error;


pub mod left_threaded_avl_tree;
pub mod datastore;

pub mod rustix_backend;

pub mod persistencer;

pub mod rustix_event_shop;

pub mod errors;

pub mod config;


use std::collections::HashSet;
use config::StaticConfig;

pub fn build_transient_backend() -> rustix_backend::RustixBackend<persistencer::TransientPersister>
{
    return build_transient_backend_with(20, 20);
}

pub fn build_transient_backend_with(
    users_per_page: u8,
    top_users: u8,
) -> rustix_backend::RustixBackend<persistencer::TransientPersister> {
    return rustix_backend::RustixBackend {
        datastore: datastore::Datastore::default(),
        persistencer: persistencer::TransientPersister {
            config: StaticConfig {
                users_per_page: users_per_page as usize,
                users_in_top_users: top_users as usize,
                top_drinks_per_user: 4,
            },
            events_stored: 0,
        },
    };
}

pub fn build_persistent_backend(
    dir: &std::path::Path,
) -> Result<rustix_backend::RustixBackend<persistencer::FilePersister>, lmdb::Error> {
    let db_flags: lmdb::DatabaseFlags = lmdb::DatabaseFlags::empty();
    let db_environment = try!(lmdb::Environment::new().set_max_dbs(1).open(dir));
    let database = try!(db_environment.create_db(None, db_flags));
    return Ok(rustix_backend::RustixBackend {
        datastore: datastore::Datastore::default(),
        persistencer: persistencer::FilePersister {
            config: config::StaticConfig {
                users_per_page: 20,
                users_in_top_users: 20,
                top_drinks_per_user: 4,
            },
            db_env: db_environment,
            db: database,
            events_stored: 0,
        },
    });
}

#[cfg(test)]
mod tests {


    extern crate tempdir;

    use rustix_backend::WriteBackend;

    use super::*;


    #[test]
    fn it_add_user() {
        let dir = tempdir::TempDir::new("temptestdir").unwrap();

        let b = build_persistent_backend(dir.as_ref());

        println!("{:?}", b);

        match b {
            Err(_) => assert!(false),
            Ok(mut backend) => {
                backend.create_user("klaus".to_string());
                assert_eq!(
                    backend.datastore.users.get(&0u32).unwrap().username,
                    "klaus".to_string()
                );
            }
        }
    }


    #[test]
    fn it_transient_add_user() {
        let mut b = build_transient_backend();
        b.create_user("klaus".to_string());
        assert_eq!(
            b.datastore.users.get(&0u32).unwrap().username,
            "klaus".to_string()
        );
    }

    #[test]
    fn it_reload_added_user() {
        /*{


            let dir = std::path::Path::new("target");

            let b = blrustix::build_persistent_backend(dir.as_ref());

            println!("{:?}", b);

            match b {
                Err(_) => assert!(false),
                Ok(mut backend) => {
                    backend.create_user("klaus".to_string());
                    assert_eq!(
                        backend.datastore.users.get(&0u32).unwrap().username,
                        "klaus".to_string()
                    );
                }
            }

        }*/




        let dir = std::path::Path::new("tests/testdata");

        {
            match build_persistent_backend(dir) {
                Err(_) => assert!(false),
                Ok(mut backend) => {
                    let x = backend.reload();
                    println!("Loaded Backend: {:?}", backend);
                    assert_eq!(x.unwrap(), 1);
                    assert_eq!(
                        backend.datastore.users.get(&0u32).unwrap().username,
                        "klaus".to_string()
                    );
                }
            }
        }
    }

}
