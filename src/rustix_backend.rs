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
    fn create_bill(&mut self, timestamp: u32, user_ids: UserGroup, comment: String) -> bool;
    fn create_item(&mut self, itemname: String, price_cents: u32, category: Option<String>)
        -> bool;
    fn create_user(&mut self, username: String) -> bool;

    fn delete_user(&mut self, user_id: u32) -> bool;
    fn delete_item(&mut self, item_id: u32) -> bool;

    fn purchase(&mut self, user_id: u32, item_id: u32, millis_timestamp: i64) -> bool;

    fn undo_purchase(&mut self, user_id: u64) -> bool;

    fn reload(&mut self) -> Result<u32, persistencer::RustixError>;
}


impl<T> WriteBackend for RustixBackend<T>
where
    T: persistencer::Persistencer + persistencer::LMDBPersistencer,
{
    fn create_bill(&mut self, timestamp: u32, user_ids: UserGroup, comment: String) -> bool {
        return self.persistencer.test_store_apply(
            &rustix_event_shop::BLEvents::CreateBill {
                timestamp: timestamp,
                user_ids: user_ids,
                comment: comment,
            },
            &mut self.datastore,
        );
    }

    fn create_item(
        &mut self,
        itemname: String,
        price_cents: u32,
        category: Option<String>,
    ) -> bool {
        return self.persistencer.test_store_apply(
            &rustix_event_shop::BLEvents::CreateItem {
                itemname: itemname,
                price_cents: price_cents,
                category: category,
            },
            &mut self.datastore,
        );
    }

    fn create_user(&mut self, username: String) -> bool {
        return self.persistencer.test_store_apply(
            &rustix_event_shop::BLEvents::CreateUser { username: username },
            &mut self.datastore,
        );
    }

    fn delete_user(&mut self, user_id: u32) -> bool {
        return self.persistencer.test_store_apply(
            &rustix_event_shop::BLEvents::DeleteUser { user_id: user_id },
            &mut self.datastore,
        );
    }

    fn delete_item(&mut self, item_id: u32) -> bool {
        return self.persistencer.test_store_apply(
            &rustix_event_shop::BLEvents::DeleteItem { item_id: item_id },
            &mut self.datastore,
        );
    }

    fn purchase(&mut self, user_id: u32, item_id: u32, millis_timestamp: i64) -> bool {
        return self.persistencer.test_store_apply(
            &rustix_event_shop::BLEvents::MakeSimplePurchase {
                user_id: user_id,
                item_id: item_id,
                timestamp: millis_timestamp,
            },
            &mut self.datastore,
        );
    }
    fn reload(&mut self) -> Result<u32, persistencer::RustixError> {
        return self.persistencer.reload_from_filepath(&mut self.datastore);
    }
    fn undo_purchase(&mut self, user_id: u64) -> bool {
        unimplemented!() //TODO: implement
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
    use datastore::UserGroup::AllUsers;
    use suffix_rs::KDTree;

    use rustix_backend::WriteBackend;

    fn build_test_backend() -> RustixBackend<persistencer::TransientPersister> {
        return RustixBackend {
            datastore: datastore::Datastore::default(),
            persistencer: persistencer::TransientPersister::default(),
        };
    }

    #[test]
    fn simple_create_user_on_backend() {
        let mut backend = build_test_backend();
        backend.create_user("klaus".to_string());
        println!("{:?}", backend);
        assert_eq!(backend.datastore.users.len(), 1);
        assert_eq!(backend.datastore.user_id_counter, 1);
        assert_eq!(
            backend.datastore.users.get(&0).unwrap().username,
            "klaus".to_string()
        );
    }

    #[test]
    fn simple_create_item_on_backend() {
        let mut backend = build_test_backend();
        backend.create_item("beer".to_string(), 95, Some("Alcohol".to_string()));
        backend.create_item("soda".to_string(), 75, None);
        assert_eq!(backend.datastore.items.len(), 2);
        assert_eq!(backend.datastore.item_id_counter, 2);
        assert_eq!(
            backend.datastore.items.get(&0).unwrap().name,
            "beer".to_string()
        );
        assert_eq!(
            backend.datastore.items.get(&1).unwrap().name,
            "soda".to_string()
        );
        assert_eq!(
            backend
                .datastore
                .items
                .get(&0)
                .unwrap()
                .category
                .clone()
                .unwrap(),
            "Alcohol".to_string()
        );
        assert_eq!(backend.datastore.items.get(&1).unwrap().cost_cents, 75);
        assert_eq!(backend.datastore.categories.len(), 1);
    }

    #[test]
    fn simple_delete_item() {
        let mut backend = build_test_backend();
        backend.create_item("beer".to_string(), 95, Some("Alcohol".to_string()));
        backend.create_item("soda".to_string(), 75, None);
        assert_eq!(backend.datastore.items.len(), 2);
        assert_eq!(backend.datastore.item_id_counter, 2);
        assert_eq!(
            backend.datastore.items.get(&0).unwrap().name,
            "beer".to_string()
        );
        assert_eq!(
            backend.datastore.items.get(&1).unwrap().name,
            "soda".to_string()
        );
        assert_eq!(
            backend
                .datastore
                .items
                .get(&0)
                .unwrap()
                .category
                .clone()
                .unwrap(),
            "Alcohol".to_string()
        );
        assert_eq!(backend.datastore.items.get(&1).unwrap().cost_cents, 75);
        assert_eq!(backend.datastore.categories.len(), 1);
        backend.delete_item(1);
        assert_eq!(backend.datastore.items.len(), 1);
        assert_eq!(backend.datastore.item_id_counter, 2);
        assert_eq!(
            backend.datastore.items.get(&0).unwrap().name,
            "beer".to_string()
        );
        assert_eq!(
            backend
                .datastore
                .items
                .get(&0)
                .unwrap()
                .category
                .clone()
                .unwrap(),
            "Alcohol".to_string()
        );
        assert_eq!(backend.datastore.categories.len(), 1);
    }


    #[test]
    fn simple_delete_user() {
        let mut backend = build_test_backend();
        backend.create_user("klaus".to_string());
        assert_eq!(backend.datastore.users.len(), 1);
        assert_eq!(backend.datastore.user_id_counter, 1);
        assert_eq!(
            backend.datastore.users.get(&0).unwrap().username,
            "klaus".to_string()
        );
        backend.delete_user(0);
        assert_eq!(backend.datastore.users.len(), 0);
        assert_eq!(backend.datastore.user_id_counter, 1);
    }


    #[test]
    fn simple_purchase() {
        let mut backend = build_test_backend();
        backend.persistencer.config.users_in_top_users = 1usize;

        //create two users
        backend.create_user("klaus".to_string());
        backend.create_user("dieter".to_string());

        //create one item
        backend.create_item("beer".to_string(), 135u32, Some("Alcoholics".to_string()));

        //make first purchase by A

        println!(
            "Beginning simple purchase test with datastore={:?}",
            backend.datastore
        );
        assert_eq!(backend.purchase(0, 0, 12345678i64), false);
        assert_eq!(backend.datastore.purchases.len(), 1);
        assert_eq!(backend.datastore.top_users.len(), 1);
        assert_eq!(backend.datastore.top_users.get(&0).unwrap(), &0u32);

        //make second purchase by B

        assert_eq!(backend.purchase(1, 0, 12345878i64), false);
        assert_eq!(backend.datastore.purchases.len(), 2);
        assert_eq!(backend.datastore.top_users.len(), 1);
        assert_eq!(backend.datastore.top_users.get(&0).unwrap(), &0u32);

        //make third purchase by B
        backend.purchase(1, 0, 12347878i64);

        //should now be A > B and all data should be correct
        assert_eq!(backend.datastore.purchases.len(), 3);
        assert_eq!(backend.datastore.top_users.len(), 1);

        println!(
            "Ending simple purchase test with datastore={:?}",
            backend.datastore
        );
        assert_eq!(backend.datastore.top_users.get(&1).unwrap(), &1u32);
        assert_eq!(
            backend
                .datastore
                .top_drinks_per_user
                .get(&0)
                .unwrap()
                .get(&0u32)
                .unwrap(),
            &0u32
        );
        assert_eq!(
            backend
                .datastore
                .top_drinks_per_user
                .get(&1)
                .unwrap()
                .get(&0u32)
                .unwrap(),
            &0u32
        );
    }

    #[test]
    fn simple_create_bill() {
        let mut backend = build_test_backend();
        //create two users, create three items, make 1 user purchase 2 items but not the third
        backend.create_user("user a".to_string());
        backend.create_user("user b".to_string());
        backend.create_item("item 1".to_string(), 45, None);
        backend.create_item("item 2".to_string(), 55, Some("category a".to_string()));
        backend.create_item("item 3".to_string(), 75, Some("category b".to_string()));


        {
            let a = backend.datastore.users_suffix_tree.search("user");
            let b = backend.datastore.users_suffix_tree.search("user a");
            let c = backend.datastore.users_suffix_tree.search("");

            assert_eq!(a.len(), 2);
            assert_eq!(b.len(), 1);
            assert_eq!(c.len(), 2);
        }


        backend.purchase(0, 0, 10);
        backend.purchase(0, 1, 20);
        backend.purchase(0, 0, 30);

        assert_eq!(
            backend
                .datastore
                .balance_cost_per_user
                .get(&0)
                .unwrap()
                .get(&0)
                .unwrap(),
            &90u32
        );

        assert_eq!(
            backend
                .datastore
                .balance_cost_per_user
                .get(&0)
                .unwrap()
                .get(&1)
                .unwrap(),
            &55u32
        );


        assert_eq!(
            backend.datastore.balance_cost_per_user.get(&1).is_none(),
            true
        );


        //create a bill
        backend.create_bill(100, AllUsers, "remark of bill".to_string());

        //control that current balance is down to zero for all users

        assert_eq!(
            backend.datastore.balance_cost_per_user.get(&0).is_none(),
            true
        );


        assert_eq!(
            backend.datastore.balance_cost_per_user.get(&1).is_none(),
            true
        );


        //control that bill contains correct data
        assert_eq!(
            backend
                .datastore
                .bills
                .get(0)
                .unwrap()
                .sum_of_cost_hash_map
                .get(&0)
                .unwrap()
                .get(&0)
                .unwrap(),
            &90u32
        );
        assert_eq!(
            backend
                .datastore
                .bills
                .get(0)
                .unwrap()
                .sum_of_cost_hash_map
                .get(&1)
                .unwrap()
                .is_empty(),
            true
        );
        assert_eq!(
            backend
                .datastore
                .bills
                .get(0)
                .unwrap()
                .count_hash_map
                .get(&0)
                .unwrap()
                .get(&1)
                .unwrap(),
            &1u32
        );
        assert_eq!(
            backend
                .datastore
                .bills
                .get(0)
                .unwrap()
                .count_hash_map
                .get(&0)
                .unwrap()
                .get(&1)
                .unwrap(),
            &1u32
        );
        assert_eq!(
            backend.datastore.bills.get(0).unwrap().timestamp_seconds,
            100u32
        );
        assert_eq!(backend.datastore.bills.get(0).unwrap().users, AllUsers);
        assert_eq!(
            backend.datastore.bills.get(0).unwrap().comment,
            "remark of bill".to_string()
        );


        //add another purchase and assert that bill didn't change
        backend.purchase(0, 0, 110);
        backend.purchase(1, 2, 120);
        backend.purchase(1, 0, 130);

        assert_eq!(
            backend
                .datastore
                .bills
                .get(0)
                .unwrap()
                .sum_of_cost_hash_map
                .get(&0)
                .unwrap()
                .get(&0)
                .unwrap(),
            &90u32
        );
        assert_eq!(
            backend
                .datastore
                .bills
                .get(0)
                .unwrap()
                .sum_of_cost_hash_map
                .get(&1)
                .unwrap()
                .is_empty(),
            true
        );
        assert_eq!(
            backend
                .datastore
                .bills
                .get(0)
                .unwrap()
                .count_hash_map
                .get(&0)
                .unwrap()
                .get(&1)
                .unwrap(),
            &1u32
        );
        assert_eq!(
            backend
                .datastore
                .bills
                .get(0)
                .unwrap()
                .count_hash_map
                .get(&0)
                .unwrap()
                .get(&1)
                .unwrap(),
            &1u32
        );
        assert_eq!(
            backend.datastore.bills.get(0).unwrap().timestamp_seconds,
            100u32
        );
        assert_eq!(backend.datastore.bills.get(0).unwrap().users, AllUsers);
        assert_eq!(
            backend.datastore.bills.get(0).unwrap().comment,
            "remark of bill".to_string()
        );
    }
}
