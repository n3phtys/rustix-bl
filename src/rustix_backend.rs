// TODO: define interface and translate into event or getter hierarchy
// TODO: keep config and datastore

use datastore;
use datastore::UserGroup;
use persistencer;

pub struct RustixBackend {
    pub datastore: datastore::Datastore,
    pub persistencer: persistencer::Persistencer,
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
    fn create_bill(timestamp: u32, user_ids: UserGroup, comment: String) -> ();
    fn create_item(itemname: String, price_cents: u32, category: Option<String>) -> ();
    fn create_user(username: String) -> ();

    fn delete_user(user_id: u32) -> ();
    fn delete_item(item_id: u32) -> ();

    fn purchase(user_id: u32, item_id: u32, timestamp: u32) -> ();
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

impl ReadBackend for RustixBackend {

}

impl WriteBackend for RustixBackend {
    fn create_bill(timestamp: u32, user_ids: UserGroup, comment: String) -> () {
        unimplemented!()
    }

    fn create_item(itemname: String, price_cents: u32, category: Option<String>) -> () {
        unimplemented!()
    }

    fn create_user(username: String) -> () {
        unimplemented!()
    }

    fn delete_user(user_id: u32) -> () {
        unimplemented!()
    }

    fn delete_item(item_id: u32) -> () {
        unimplemented!()
    }

    fn purchase(user_id: u32, item_id: u32, timestamp: u32) -> () {
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

    use rustix_backend::WriteBackend;
    use rustix_backend::ReadBackend;

    fn build_test_backend() -> RustixBackend {
        return RustixBackend {
            datastore: datastore::Datastore{},
            persistencer: persistencer::TransientPersister{events_stored : 0u32},
        }
    }

    #[test]
    fn simple_create_user_on_backend() {
        let backend = build_test_backend();

    }
}