// An attribute to hide warnings for unused code.
#![allow(dead_code)]
#![allow(unused_parens)]
#![allow(unused_variables)]



use datastore::Datastore;

use serde_json;
use serde_json::Error;

pub struct SimplePersistencer {
    filepath: Option<String>,
}


trait Persistencer {
    fn test_store_apply<T: Event>(&self, event: &T) -> bool;
    fn reload_from_filepath<T: Event>(&self, event: &T) -> u32; //returns number of events loaded
}

impl Persistencer for SimplePersistencer {
    fn test_store_apply<T: Event>(&self, event: &T) -> bool {
        unimplemented!()
    }

    fn reload_from_filepath<T: Event>(&self, event: &T) -> u32 {
        unimplemented!()
    }
}

trait Event {
    fn load(rawdata: &[u8] ) -> Self;
    fn store(&self) -> &[u8] ;
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
    fn load(rawdata: &[u8]) -> Self {
            unimplemented!()
    }

    fn store(&self) -> &[u8] {
        unimplemented!()
    }

    fn can_be_applied(&self, store: &Datastore) -> bool {
        unimplemented!()
    }

    fn apply(&self, store: &mut Datastore) -> () {
        unimplemented!()
    }
}

trait EventList {
    fn load(rawdata: Vec<u8>) -> Self;
    fn store(&self) ->  String;
}

impl EventList for Vec<BLEvents> {
    fn load(rawdata: Vec<u8>) -> Self {
        return serde_json::from_str(&String::from_utf8(rawdata).unwrap()).unwrap();
    }

    fn store(&self) -> String {
        let json = serde_json::to_string(self).unwrap();
        return json;
    }
}

#[cfg(test)]
mod tests {

    use event_source_persistence::EventList;
    use event_source_persistence::BLEvents;
    use serde_json;

    #[test]
    fn events_serialize_and_deserialize_raw() {
        let v = vec![
            BLEvents::CreateItem {itemname: "beer".to_string(), price_euros : 0u8, price_cents: 95u8,},
            BLEvents::CreateItem {itemname: "beer 2".to_string(), price_euros : 0u8, price_cents: 95u8,},
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
            BLEvents::CreateItem {itemname: "beer".to_string(), price_euros : 0u8, price_cents: 95u8,},
            BLEvents::CreateItem {itemname: "beer 2".to_string(), price_euros : 0u8, price_cents: 95u8,},
            BLEvents::DeleteItem {item_id: 2u32,},
            BLEvents::CreateUser {username: "klaus".to_string(),},
            BLEvents::MakeSimplePurchase {item_id: 1u32, user_id: 1u32,timestamp: 123456789u32,}
        ];

        // Serialize it to a JSON string.
        let json_bytes = v.store().as_bytes().to_vec();
        println!("{:?}", json_bytes);
        let reparsed_content : Vec<BLEvents> = Vec::load(json_bytes);
        println!("{:#?}", reparsed_content);
        assert_eq!(reparsed_content, v);
    }
}