// An attribute to hide warnings for unused code.
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#[macro_use]
pub extern crate derive_builder;
pub extern crate serde;
pub extern crate serde_json;
pub extern crate lmdb;

#[macro_use]
pub extern crate serde_derive;
pub extern crate bincode;

#[macro_use]
pub extern crate quick_error;

use std::collections::HashSet;



pub mod left_threaded_avl_tree;
pub mod datastore;

pub mod rustix_backend;

pub mod persistencer;

pub mod rustix_event_shop;

pub mod errors;

pub mod config;

pub fn build_transient_backend() -> rustix_backend::RustixBackend<persistencer::TransientPersister> {
    return build_transient_backend_with(20, 20);
}

pub fn build_transient_backend_with(users_per_page: u8, top_users: u8) -> rustix_backend::RustixBackend<persistencer::TransientPersister> {
    return rustix_backend::RustixBackend {
        datastore: datastore::Datastore {
            users: Vec::new(),
            items: Vec::new(),
            user_id_counter: 0,
            item_id_counter: 0,
            categories: HashSet::new(),
        },
        persistencer: persistencer::TransientPersister{ events_stored: 0 },
    };
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
