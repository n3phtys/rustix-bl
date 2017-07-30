// An attribute to hide warnings for unused code.
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#[macro_use] extern crate derive_builder;
extern crate serde;
extern crate serde_json;
extern crate lmdb;

#[macro_use] extern crate serde_derive;
extern crate bincode;

#[macro_use] extern crate quick_error;




mod itemstorage;
mod left_threaded_avl_tree;
mod event_source_persistence;
mod datastore;

mod rustix_backend;

mod persistencer;

mod rustix_event_shop;

mod errors;

mod config;

#[derive(Default, Builder, Debug)]
#[builder(setter(into))]
struct Channel {
    token: i32,
    special_info: i32,
    // .. a whole bunch of other fields ..
}


fn main() {
    println!("Hello, world!");

    itemstorage::test();

    // builder pattern, go, go, go!...
    let ch = ChannelBuilder::default()
        .special_info(42u8)
        .token(19124)
        .build()
        .unwrap();
    println!("{:?}", ch);




    rustix::test();

    ldmbtest::write_to_lmdb();
}


mod ldmbtest {
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

    pub fn write_to_lmdb() {
        let path = Path::new("target");
        let env = &Environment::new()
                        .open(path).unwrap();

        let db : Database = env.create_db(None, DatabaseFlags::empty()).unwrap();

        let mut rw_transaction: RwTransaction = RwTransaction::new(env).unwrap();
        let tx_flags: WriteFlags = WriteFlags::empty();
        for i in 1..100 {
            let key1 = transform_u32_to_array_of_u8(0u32 + i);
            let data1 = transform_abc_to_array_of_u8(((i as u8) % (122u8 - 65u8) + 65u8) as u8);
            let result = rw_transaction.put(db, &key1, &data1, tx_flags );

        }
        let datajson = serde_json::to_string(&Data::A {astr: "abc".to_string(),  bstr: "def".to_string()}).unwrap();
        let result = rw_transaction.put(db, &transform_u32_to_array_of_u8(0u32 + 101), &datajson, tx_flags );
        rw_transaction.commit().unwrap();

        let raw_ro_transaction = RoTransaction::new(env).unwrap();
        {
            let mut read_transaction: RoCursor = RoCursor::new(&raw_ro_transaction, db).unwrap();

            for keyvalue in read_transaction.iter_start() {
                let (key, value) = keyvalue;
                let string = str::from_utf8(value).unwrap();
                println!("{:?}", string);
            }
        }
        raw_ro_transaction.commit().unwrap();
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    enum Data {
        A{astr: String, bstr: String},
        B{astr: String, bstr: String},
        C{i: u32},
        D,
    }


    fn transform_u32_to_array_of_u8(x:u32) -> [u8;4] {
        let b1 : u8 = ((x >> 24) & 0xff) as u8;
        let b2 : u8 = ((x >> 16) & 0xff) as u8;
        let b3 : u8 = ((x >> 8) & 0xff) as u8;
        let b4 : u8 = (x & 0xff) as u8;
        return [b1, b2, b3, b4]
    }

    fn transform_abc_to_array_of_u8(x:u8) -> [u8;3] {
        return [x,x+1u8,x+2u8]
    }
}


#[macro_use]
mod rustix {

    #[derive(Default, Builder, Debug)]
    #[builder(setter(into))]
    struct User {
        username: String,
        external_user_id: u32,
        user_id: u32,
        subuser_to: Option<u32>,
        is_billed: bool,
    }



    #[derive(Default, Builder, Debug)]
    #[builder(setter(into))]
    struct Item {
        name: String,
        item_id: u32,
        cost_euros: u8,
        cost_cents: u8,
    }


    pub fn test() {

        let x = UserBuilder::default()
            .external_user_id(19124u32)
            .user_id(1234u32)
            .subuser_to(None)
            .is_billed(true)
            .username("klaus")
            .build()
        ;
        println!("{:?}", x);

        let y = ItemBuilder::default()
            .name("cool item")
            .cost_euros(42u8)
            .cost_cents(13u8)
            .item_id(19124u32)
            .build()
        ;
        println!("{:?}", y);
    }
}