// TODO: define interface and translate into event or getter hierarchy
// TODO: keep config and datastore

use datastore;
use datastore::UserGroup;
use persistencer;
use rustix_event_shop;
use persistencer::LMDBPersistencer;
use persistencer::Persistencer;

#[derive(Debug)]
pub struct RustixBackend<T: persistencer::Persistencer + persistencer::LMDBPersistencer> {
    pub datastore: datastore::Datastore,
    pub persistencer: T,
}
/*

    CreateItem{itemname: String, price_cents: u32, category: Option<String>},
    CreateUser{username: String},
    DeleteItem{item_id: u32},
    DeleteUser{user_id: u32},
    MakeSimplePurchase{user_id: u32, item_id: u32, timestamp: u32},
    CreateBill{timestamp: u32, user_ids: UserGroup, comment: String},
*/

pub trait WriteBackend {
    fn create_bill(&mut self, timestamp: u32, user_ids: UserGroup, comment: String) -> ();
    fn create_item(&mut self, itemname: String, price_cents: u32, category: Option<String>) -> ();
    fn create_user(&mut self, username: String) -> ();

    fn delete_user(&mut self, user_id: u32) -> ();
    fn delete_item(&mut self, item_id: u32) -> ();

    fn purchase(&mut self, user_id: u32, item_id: u32, timestamp: u32) -> ();
}

pub trait ReadBackend {
    //fn get_active_categories() -> &[String];

    //TODO: hashmap user

    //TODO: hashmap bill (including 'empty' bill that is not yet created)

    //TODO: hashmap item

    //TODO: top users list

    //TODO: paginated users

    //TODO: categorized items with Option<String> as key for hashmap

    //TODO: get top items of user
}

impl <T> ReadBackend for RustixBackend<T> where T: persistencer::Persistencer+persistencer::LMDBPersistencer{

}


impl <T> WriteBackend for RustixBackend<T> where T: persistencer::Persistencer+persistencer::LMDBPersistencer {
    fn create_bill(&mut self, timestamp: u32, user_ids: UserGroup, comment: String) -> () {
        unimplemented!()
    }

    fn create_item(&mut self, itemname: String, price_cents: u32, category: Option<String>) -> () {
        self.persistencer.test_store_apply(&rustix_event_shop::BLEvents::CreateItem{itemname: itemname, price_cents: price_cents, category: category}, &mut self.datastore);
    }

    fn create_user(&mut self, username: String) -> () {
        self.persistencer.test_store_apply(&rustix_event_shop::BLEvents::CreateUser {username: username}, &mut self.datastore);
    }

    fn delete_user(&mut self, user_id: u32) -> () {
        unimplemented!()
    }

    fn delete_item(&mut self, item_id: u32) -> () {
        unimplemented!()
    }

    fn purchase(&mut self, user_id: u32, item_id: u32, timestamp: u32) -> () {
        unimplemented!()
    }
}


//TODO: write full test suite in here, testing without file persistencer



#[cfg(test)]
mod tests {
    use rustix_event_shop::BLEvents;
    use serde_json;
    use rustix_backend::RustixBackend;
    use std;
    use datastore;
    use persistencer;
    use std::collections::HashSet;

    use rustix_backend::WriteBackend;
    use rustix_backend::ReadBackend;

    fn build_test_backend() -> RustixBackend<persistencer::TransientPersister> {
        return RustixBackend {
            datastore: datastore::Datastore{items: Vec::new(), users: Vec::new(), user_id_counter: 0, item_id_counter:0, categories: HashSet::new() },
            persistencer: persistencer::TransientPersister{events_stored : 0u32},
        }
    }

    #[test]
    fn simple_create_user_on_backend() {
        let mut backend = build_test_backend();
        backend.create_user("klaus".to_string());
        assert_eq!(backend.datastore.users.len(), 1);
        assert_eq!(backend.datastore.user_id_counter, 1);
        assert_eq!(backend.datastore.users.get(0).unwrap().username, "klaus".to_string());
    }

    #[test]
    fn simple_create_item_on_backend() {
        let mut backend = build_test_backend();
        backend.create_item("beer".to_string(), 95, Some("Alcohol".to_string()));
        backend.create_item("soda".to_string(), 75, None);
        assert_eq!(backend.datastore.items.len(), 2);
        assert_eq!(backend.datastore.item_id_counter, 2);
        assert_eq!(backend.datastore.items.get(0).unwrap().name, "beer".to_string());
        assert_eq!(backend.datastore.items.get(1).unwrap().name, "soda".to_string());
        assert_eq!(backend.datastore.items.get(0).unwrap().category.clone().unwrap(), "Alcohol".to_string());
        assert_eq!(backend.datastore.items.get(1).unwrap().cost_cents, 75);
        assert_eq!(backend.datastore.categories.len(), 1);
    }
}