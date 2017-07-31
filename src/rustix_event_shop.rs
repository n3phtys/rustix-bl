// An attribute to hide warnings for unused code.
#![allow(dead_code)]
#![allow(unused_parens)]
#![allow(unused_variables)]

use datastore::Datastore;
use datastore::UserGroup;

use serde_json;
use std;
use serde_json::Error;
use datastore;
use std::collections::HashSet;



pub trait Event {
    fn can_be_applied(&self, store: &Datastore) -> bool;
    fn apply(&self, store: &mut Datastore) -> () ;
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum BLEvents {
    CreateItem{itemname: String, price_cents: u32, category: Option<String>},
    CreateUser{username: String},
    DeleteItem{item_id: u32},
    DeleteUser{user_id: u32},
    MakeSimplePurchase{user_id: u32, item_id: u32, timestamp: u32},
    CreateBill{timestamp: u32, user_ids: UserGroup, comment: String},
}



impl Event for BLEvents {

    fn can_be_applied(&self, store: &Datastore) -> bool {
        return match self {
            &BLEvents::CreateItem{ref itemname, price_cents, ref category} => true,
            &BLEvents::CreateUser{ref username} => true,
            &BLEvents::CreateBill{timestamp, ref user_ids, ref comment} => unimplemented!(),//TODO:
            &BLEvents::DeleteItem{item_id} => unimplemented!(), //TODO:
            &BLEvents::DeleteUser{user_id} => unimplemented!(), //TODO:
            &BLEvents::MakeSimplePurchase{user_id, item_id, timestamp} => unimplemented!(),//TODO:
        }
    }

    fn apply(&self, store: &mut Datastore) -> () {
        return match self {
            &BLEvents::CreateItem{ref itemname, price_cents, ref category} => {
                let id = store.item_id_counter;
                for cat in category.iter() {
                    store.categories.insert(cat.to_string());
                }
                store.items.push(datastore::Item{name: itemname.to_string(), item_id: id, cost_cents: price_cents, category: category.clone()});
                store.item_id_counter = id + 1u32;
            },
            &BLEvents::CreateUser{ref username} => {
                let id = store.user_id_counter;
                store.users.push(datastore::User{username: username.to_string(), user_id: id, is_billed: true});
                store.user_id_counter = id + 1u32;
            },
            &BLEvents::CreateBill{timestamp, ref user_ids, ref comment} => unimplemented!(),//TODO:
            &BLEvents::DeleteItem{item_id} => unimplemented!(),//TODO:
            &BLEvents::DeleteUser{user_id} => unimplemented!(),//TODO:
            &BLEvents::MakeSimplePurchase{user_id, item_id, timestamp} => unimplemented!(),//TODO:
        }
    }
}

//TODO: finish declaring all possible events here





#[cfg(test)]
mod tests {
    use rustix_event_shop::BLEvents;
    use serde_json;
    use std;

    #[test]
    fn events_serialize_and_deserialize_raw() {
        let v = vec![
            BLEvents::CreateItem {itemname: "beer".to_string(), price_cents: 95u32, category: None},
            BLEvents::CreateItem {itemname: "beer 2".to_string(), price_cents: 95u32, category: None},
            BLEvents::DeleteItem {item_id: 2u32,},
            BLEvents::CreateUser {username: "klaus".to_string(),},
            BLEvents::MakeSimplePurchase {item_id: 1u32, user_id: 1u32,timestamp: 123456789u32,}
        ];

        // Serialize it to a JSON string.
        let json = serde_json::to_string(&v).unwrap();
        println!("{}", json);
        let reparsed_content : Vec<BLEvents> = serde_json::from_str(&json).unwrap();
        println!("{:#?}", reparsed_content);
        assert_eq!(reparsed_content, v);
    }

    #[test]
    fn events_serialize_and_deserialize_packed() {
        let v = vec![
            BLEvents::CreateItem {itemname: "beer".to_string(), price_cents: 95u32, category: None,},
            BLEvents::CreateItem {itemname: "beer 2".to_string(), price_cents: 95u32, category: None,},
            BLEvents::DeleteItem {item_id: 2u32,},
            BLEvents::CreateUser {username: "klaus".to_string(),},
            BLEvents::MakeSimplePurchase {item_id: 1u32, user_id: 1u32,timestamp: 123456789u32,}
        ];

        // Serialize it to a JSON string.
        let json_bytes = serde_json::to_string(&v).unwrap().as_bytes().to_vec();
        println!("{:?}", json_bytes);
        let reparsed_content : Vec<BLEvents> = serde_json::from_str(std::str::from_utf8(json_bytes.as_ref()).unwrap()).unwrap();
        println!("{:#?}", reparsed_content);
        assert_eq!(reparsed_content, v);
    }
}