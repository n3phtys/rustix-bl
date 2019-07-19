// An attribute to hide warnings for unused code.
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#[macro_use]
extern crate typescriptify_derive;
extern crate typescriptify;

#[macro_use]
pub extern crate derive_builder;
pub extern crate lmdb;

pub extern crate serde;
pub extern crate serde_json;
pub extern crate serde_yaml;

pub extern crate byteorder;


pub extern crate suffix_rs;

pub extern crate unidecode;

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

pub fn build_transient_backend() -> rustix_backend::RustixBackend
{
    return build_transient_backend_with(20, 20);
}

pub fn build_transient_backend_with(
    users_per_page: u8,
    top_users: u8,
) -> rustix_backend::RustixBackend {
    let mut config = StaticConfig::default();
    config.users_per_page = users_per_page as usize;
    config.users_in_top_users = top_users as usize;
    config.top_drinks_per_user = 4;

    return rustix_backend::RustixBackend {
        datastore: datastore::Datastore::default(),
        persistencer: persistencer::FilePersister::new(config).unwrap(),
    };
}

pub fn build_persistent_backend(dir: &std::path::Path) -> rustix_backend::RustixBackend {
    let config = StaticConfig::default_persistence(dir.to_str().unwrap());

    return rustix_backend::RustixBackend {
        datastore: datastore::Datastore::default(),
        persistencer: persistencer::FilePersister::new(config).unwrap(),
    };
}


#[cfg(test)]
mod tests {


    extern crate tempdir;

    use rustix_backend::WriteBackend;

    use super::*;


    #[test]
    fn it_add_user() {
        let dir = tempdir::TempDir::new("temptestdir").unwrap();

        let mut backend = build_persistent_backend(dir.as_ref());

        println!("{:?}", backend);
 {
                backend.create_user("klaus".to_string());
                assert_eq!(
                    backend.datastore.users.get(&0u32).unwrap().username,
                    "klaus".to_string()
                );
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


}
