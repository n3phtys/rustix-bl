// An attribute to hide warnings for unused code.
#![allow(dead_code)]
#![allow(unused_parens)]
#![allow(unused_variables)]

use datastore::Datastore;

use serde_json;
use serde_json::Error;


pub trait Event {
    fn can_be_applied(&self, store: &Datastore) -> bool;
    fn apply(&self, store: &mut Datastore) -> () ;
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum BLEvents {
    CreateItem{itemname: String, price_euros: u8, price_cents: u8,},
    CreateUser{username: String},
    DeleteItem{item_id: u32},
    DeleteUser{user_id: u32},
    MakeSimplePurchase{user_id: u32, item_id: u32, timestamp: u32},
}


impl Event for BLEvents {

    fn can_be_applied(&self, store: &Datastore) -> bool {
        unimplemented!()
    }

    fn apply(&self, store: &mut Datastore) -> () {
        unimplemented!()
    }
}

//TODO: finish declaring all possible events here