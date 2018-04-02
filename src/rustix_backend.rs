use datastore;
use datastore::UserGroup;
use persistencer;
use rustix_event_shop;
use persistencer::LMDBPersistencer;
use persistencer::Persistencer;
use serde_json;
use std;
use std::fs::File;
use std::io::prelude::*;
use datastore::Datastore;

#[derive(Debug)]
pub struct RustixBackend {
    pub datastore: datastore::Datastore,
    pub persistencer: persistencer::FilePersister,
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

    fn apply(&mut self, event: &rustix_event_shop::BLEvents) -> bool;

    fn snapshot(&mut self) -> Option<u64>;

    fn load_snapshot(&mut self) -> Option<u64>;

    fn create_bill(&mut self, timestamp_from: i64, timestamp_to: i64, user_ids: UserGroup, comment: String) -> bool;
    fn create_item(&mut self, itemname: String, price_cents: u32, category: Option<String>)
                   -> bool;
    fn create_user(&mut self, username: String) -> bool;
    fn update_item(&mut self, item_id: u32, itemname: String, price_cents: u32, category: Option<String>)
                   -> bool;
    fn update_user(&mut self, user_id: u32, username: String, is_billed: bool, is_highlighted: bool, external_user_id: Option<String>) -> bool;

    fn delete_user(&mut self, user_id: u32) -> bool;
    fn delete_item(&mut self, item_id: u32) -> bool;

    fn purchase(&mut self, user_id: u32, item_id: u32, millis_timestamp: i64) -> bool;

    fn special_purchase(&mut self, user_id: u32, special_name: String, millis_timestamp: i64) -> bool;

    fn cart_purchase(&mut self, user_id: u32, specials: Vec<String>, item_ids: Vec<u32>, millis_timestamp: i64) -> bool;

    fn ffa_purchase(&mut self, ffa_id: u64, item_id: u32, millis_timestamp: i64) -> bool;

    fn create_ffa(&mut self, allowed_categories : Vec<String>,
                         allowed_drinks : Vec<u32>,
                         allowed_number_total : u16,
                         text_message : String,
                         created_timestamp : i64,
                         donor : u32) -> bool;

    fn create_free_budget(&mut self, cents_worth_total : u64,
                          text_message : String,
                          created_timestamp : i64,
                          donor : u32,
                          recipient : u32) -> bool;

    fn create_free_count(&mut self, allowed_categories : Vec<String>,
                         allowed_drinks : Vec<u32>,
                         allowed_number_total : u16,
                         text_message : String,
                         created_timestamp : i64,
                         donor : u32,
                         recipient : u32) -> bool;

    fn undo_purchase(&mut self, unique_id: u64) -> bool;

    fn reload(&mut self) -> Result<u64, persistencer::RustixError>;
}


impl WriteBackend for RustixBackend {
    fn create_bill(&mut self, timestamp_from: i64, timestamp_to: i64, user_ids: UserGroup, comment: String) -> bool {
        return self.persistencer.test_store_apply(
            &rustix_event_shop::BLEvents::CreateBill {
                timestamp_from: timestamp_from,
                timestamp_to: timestamp_to,
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


    fn reload(&mut self) -> Result<u64, persistencer::RustixError> {
        let counter = self.load_snapshot();
        return self.persistencer.reload_from_filepath(&mut self.datastore);
    }
    fn undo_purchase(&mut self, unique_id: u64) -> bool {
        return self.persistencer.test_store_apply(
            &rustix_event_shop::BLEvents::UndoPurchase {
                unique_id: unique_id,
            },
            &mut self.datastore,
        );
    }
    fn special_purchase(&mut self, user_id: u32, special_name: String, millis_timestamp: i64) -> bool {
        return self.persistencer.test_store_apply(
            &rustix_event_shop::BLEvents::MakeSpecialPurchase {
                user_id: user_id,
                special_name: special_name,
                timestamp: millis_timestamp,
            },
            &mut self.datastore,
        );
    }

    fn ffa_purchase(&mut self, ffa_id: u64, item_id: u32, millis_timestamp: i64) -> bool {
        return self.persistencer.test_store_apply(
            &rustix_event_shop::BLEvents::MakeFreeForAllPurchase {
            ffa_id: ffa_id,
                item_id: item_id,
            timestamp: millis_timestamp,
            },
            &mut self.datastore,
        );
    }

    fn create_ffa(&mut self, allowed_categories: Vec<String>, allowed_drinks: Vec<u32>, allowed_number_total: u16, text_message: String, created_timestamp: i64, donor: u32) -> bool {
        return self.persistencer.test_store_apply(
            &rustix_event_shop::BLEvents::CreateFreeForAll {
                allowed_categories: allowed_categories,
                allowed_drinks: allowed_drinks,
                allowed_number_total: allowed_number_total,
                text_message: text_message,
                created_timestamp: created_timestamp,
                donor: donor,
            },
            &mut self.datastore,
        );
    }

    fn create_free_budget(&mut self, cents_worth_total: u64, text_message: String, created_timestamp: i64, donor: u32, recipient: u32) -> bool {

        return self.persistencer.test_store_apply(
            &rustix_event_shop::BLEvents::CreateFreeBudget {
                cents_worth_total: cents_worth_total,
                text_message: text_message,
                created_timestamp: created_timestamp,
                donor: donor,
                recipient: recipient,
            },
            &mut self.datastore,
        );
    }

    fn create_free_count(&mut self, allowed_categories: Vec<String>, allowed_drinks: Vec<u32>, allowed_number_total: u16, text_message: String, created_timestamp: i64, donor: u32, recipient: u32) -> bool {

        return self.persistencer.test_store_apply(
            &rustix_event_shop::BLEvents::CreateFreeCount {
                allowed_categories: allowed_categories,
                allowed_drinks: allowed_drinks,
                allowed_number_total: allowed_number_total,
                text_message: text_message,
                created_timestamp: created_timestamp,
                donor: donor,
                recipient: recipient,
            },
            &mut self.datastore,
        );
    }
    fn cart_purchase(&mut self, user_id: u32, specials: Vec<String>, item_ids: Vec<u32>, millis_timestamp: i64) -> bool {
        return self.persistencer.test_store_apply(
            &rustix_event_shop::BLEvents::MakeShoppingCartPurchase {
                user_id: user_id,
                specials: specials,
                item_ids: item_ids,
                timestamp: millis_timestamp,
            },
            &mut self.datastore,
        );
    }
    fn update_item(&mut self, item_id: u32, itemname: String, price_cents: u32, category: Option<String>) -> bool {
        return self.persistencer.test_store_apply(
            &rustix_event_shop::BLEvents::UpdateItem {
                item_id: item_id,
                itemname: itemname,
                price_cents: price_cents,
                category: category,
            },
            &mut self.datastore,
        );
    }

    fn update_user(&mut self, user_id: u32, username: String, is_billed: bool, is_highlighted: bool, external_user_id: Option<String>) -> bool {
        return self.persistencer.test_store_apply(
            &rustix_event_shop::BLEvents::UpdateUser {
                user_id: user_id,
                username: username,
                is_billed: is_billed,
                is_highlighted: is_highlighted,
                external_user_id: external_user_id,
            },
            &mut self.datastore,
        );
    }
    fn apply(&mut self, event: &rustix_event_shop::BLEvents) -> bool {
        return self.persistencer.test_store_apply(event, &mut self.datastore);
    }

    fn snapshot(&mut self) -> Option<u64> {
        //only if persistence layer
        if !self.persistencer.config.use_persistence {
            println!("snapshot() called, but not using persistence");
            return None;
        }

        //take path of dir: <path>/snapshot.json
        let filepath = self.persistencer.config.persistence_file_path.to_owned() + "/snapshot.json";
        println!("snapshot() called, with file = {}", &filepath);

        println!("datastore = {:?}", &self.datastore);

        //take current state and turn it into json
        match serde_json::to_string(&self.datastore) {
            Ok(json) => {

                println!("snapshot() called, writing json = {}", &json);
                //write to file
                let mut file_res = std::fs::File::create(filepath);
                match file_res {
                    Ok(mut file) => {
                        let res = file.write_all(&json.into_bytes());
                        match res {
                            Ok(e) => {
                                //if successful, return version of aggregate
                                println!("Success on snapshot()");
                                return Some(self.datastore.version);
                            },
                            Err(e) => {
                                println!("Eror writing file on snapshot(): {:?}", e);
                                return None
                            },
                        }
                    },
                    Err(e) => {
                        println!("Error opening file on snapshot(): {:?}", e);
                        return None
                    },
                }
            },
            //if failure, return None
            Err(e) => {
                println!("Error creating json on snapshot(): {:?}", e);
                return None
            },
        }
    }
    fn load_snapshot(&mut self) -> Option<u64> {
        //only if using persistence
        if !self.persistencer.config.use_persistence {
            return None;
        }

        //take <persistence_path>/snapshot.json and load it
        let filepath = self.persistencer.config.persistence_file_path.to_owned() + "/snapshot.json";

        let mut file_raw= File::open(filepath);
        if file_raw.is_err() {
            return None;
        }
        let mut file = file_raw.unwrap();
        let mut contents: String = String::new();
        if file.read_to_string(&mut contents).is_err() {
            return None
        }

        //extract datastore from json
        let ds_raw = serde_json::from_str(&contents);
        if (ds_raw.is_err()) {
            return None;
        }
        let ds: Datastore = ds_raw.unwrap();

        //write datastore to backend
        let version: u64 = ds.version;
        self.datastore = ds;

        //if successful, return counter / version
        return Some(version);
    }
}


//TODO: write full test suite in here, testing without file persistencer


#[cfg(test)]
mod tests {
    use rustix_event_shop;
    use rustix_event_shop::BLEvents;
    use serde_json;
    use rustix_backend::RustixBackend;
    use std;
    use datastore;
    use persistencer;
    use config;
    use std::collections::HashSet;
    use datastore::UserGroup::AllUsers;
    use suffix_rs::KDTree;
    use datastore::*;
    use datastore::PurchaseFunctions;
    use datastore::DatastoreQueries;
    use rustix_backend::WriteBackend;
    use rustix_event_shop::BLEvents::SetPriceForSpecial;

    fn build_test_backend() -> RustixBackend {
        let config = config::StaticConfig::default();
        return RustixBackend {
            datastore: datastore::Datastore::default(),
            persistencer: persistencer::FilePersister::new(config).unwrap(),
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
        assert_eq!(backend.datastore.items.len(), 2);
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
        assert_eq!(backend.datastore.users.len(), 1);
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

        assert_eq!(backend.purchase(1, 0, 12345888i64), false);
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


        println!("Datastore before search: {:?}", backend.datastore);

        //get purchases
        assert_eq!(backend.datastore.get_purchase(1).unwrap().get_item_id(), &0u32);
        assert_eq!(backend.datastore.get_purchase(1).unwrap().get_user_id(), &0u32);
        assert_eq!(backend.datastore.get_purchase(1).unwrap().get_unique_id(), 1u64);
        assert_eq!(backend.datastore.get_purchase(2).unwrap().get_item_id(), &0u32);
        assert_eq!(backend.datastore.get_purchase(2).unwrap().get_user_id(), &1u32);
        assert_eq!(backend.datastore.get_purchase(2).unwrap().get_unique_id(), 2u64);
        assert_eq!(backend.datastore.get_purchase(3).unwrap().get_item_id(), &0u32);
        assert_eq!(backend.datastore.get_purchase(3).unwrap().get_user_id(), &1u32);
        assert_eq!(backend.datastore.get_purchase(3).unwrap().get_unique_id(), 3u64);

        assert_eq!(backend.datastore.get_purchase(0).is_none(), true);

        assert_eq!(backend.datastore.get_purchase(4).is_none(), true);


        //test all user query
        let two_user = backend.datastore.users_searchhit_ids("");
        let one_user = backend.datastore.users_searchhit_ids("klau");
        let zero_user = backend.datastore.users_searchhit_ids("lisa");

        assert_eq!(two_user.len(), 2);
        assert_eq!(one_user.len(), 1);
        assert_eq!(zero_user.len(), 0);

        //test all item query
        let all_item = backend.datastore.items_searchhit_ids("");
        let one_item = backend.datastore.items_searchhit_ids("beer");
        let zero_item = backend.datastore.items_searchhit_ids("cola");

        assert_eq!(one_item.len(), 1);
        assert_eq!(all_item.len(), 1);
        assert_eq!(zero_item.len(), 0);

        //test top user query
        let top_two_user = backend.datastore.top_user_ids(2);
        let top_three_user = backend.datastore.top_user_ids(3);
        let top_one_user = backend.datastore.top_user_ids(1);
        let top_zero_user = backend.datastore.top_user_ids(0);

        assert_eq!(top_two_user.len(), 2);
        assert_eq!(top_three_user.len(), 2);
        assert_eq!(top_one_user.len(), 1);
        assert_eq!(top_zero_user.len(), 0);


        //test top item query

        let top_items = backend.datastore.top_item_ids(0, 2);
        let no_top_items = backend.datastore.top_item_ids(0, 0);

        assert_eq!(top_items.len(), 1);
        assert_eq!(no_top_items.len(), 0);

        //test purchase query personal (by user with id = 1)
        let lowest_time_point = 1000i64;
        let low_mid_time_point =    12345680i64;
        let mid_time_point =        12345880i64;
        let high_mid_time_poin =    12345890i64;
        let highest_time_point =    12447878i64;

        let all_personal_purchases = backend.datastore.personal_log_filtered(1, lowest_time_point, highest_time_point);
        let one_personal_purchases = backend.datastore.personal_log_filtered(1, low_mid_time_point, high_mid_time_poin);
        let zero_personal_purchases = backend.datastore.personal_log_filtered(1, low_mid_time_point, mid_time_point);

        assert_eq!(all_personal_purchases.len(), 2);
        assert_eq!(one_personal_purchases.len(), 1);
        assert_eq!(zero_personal_purchases.len(), 0);


        //test purchase query global
        let all_global_purchases = backend.datastore.global_log_filtered(lowest_time_point, highest_time_point);
        let one_global_purchases = backend.datastore.global_log_filtered(low_mid_time_point, high_mid_time_poin);
        let zero_global_purchases = backend.datastore.global_log_filtered(low_mid_time_point, mid_time_point);


        assert_eq!(all_global_purchases.len(), 3);
        assert_eq!(one_global_purchases.len(), 1);
        assert_eq!(zero_global_purchases.len(), 0);

    }

    #[test]
    fn simple_create_bill() {
        let mut backend = build_test_backend();
        //create two users, create three items, make 1 user purchase 2 items but not the third
        backend.create_user("user a".to_string());
        backend.create_user("user b".to_string());
        backend.create_user("donated_to_user".to_string());
        backend.create_item("item 1".to_string(), 45, None);
        backend.create_item("item 2".to_string(), 55, Some("category a".to_string()));
        backend.create_item("item 3".to_string(), 75, Some("category b".to_string()));


        {
            let a = backend.datastore.users_suffix_tree.search("user");
            let b = backend.datastore.users_suffix_tree.search("user a");
            let c = backend.datastore.users_suffix_tree.search("");

            assert_eq!(a.len(), 3);
            assert_eq!(b.len(), 1);
            assert_eq!(c.len(), 3);
        }


        backend.purchase(0, 0, 10);
        backend.purchase(0, 1, 20);
        backend.purchase(0, 0, 30);

        backend.create_free_budget(1000, "some budget message".to_string(), 31, 0, 2);
        backend.create_free_count(vec!["category a".to_string()], vec![0], 2, "some count message".to_string(), 32, 0, 2);

        backend.purchase(2, 0, 33);
        backend.purchase(2, 1, 34);



        backend.create_ffa(Vec::new(), vec![0, 1], 2, "my ffa message".to_string(), 35, 0);
        let ffa_id : u64 = backend.datastore.open_ffa[0].get_id();


        backend.purchase(2, 0, 36);
        backend.purchase(2, 0, 37);
        backend.purchase(2, 1, 38);
        backend.purchase(2, 0, 39);


        backend.special_purchase(0, "some special".to_string(), 40);

        assert_eq!(backend.ffa_purchase(ffa_id, 0, 50), true);
        assert_eq!(backend.ffa_purchase(ffa_id, 0, 60), true);


        assert_eq!(backend.ffa_purchase(ffa_id, 1, 70), false);

        let user_key = (0, (&backend).datastore.users.get(&0).unwrap().username.to_string());
        let user_1_key = (1, (&backend).datastore.users.get(&1).unwrap().username.to_string());
        let item_0_key = (0, (&backend).datastore.items.get(&0).unwrap().name.to_string());
        let item_1_key = (1, (&backend).datastore.items.get(&1).unwrap().name.to_string());
        println!("Testoutput = {:?}\nwith user key = {:?}\nand item key = {:?}", backend.datastore, &user_key, &item_0_key);
        assert!(backend
            .datastore
            .balance_cost_per_user
            .get(&user_key)
            .is_some());
        assert_eq!(
            backend
                .datastore
                .balance_cost_per_user
                .get(&user_key)
                .unwrap()
                .get(&item_0_key)
                .unwrap(),
            &180u32
        );

        assert_eq!(
            backend
                .datastore
                .balance_cost_per_user
                .get(&user_key)
                .unwrap()
                .get(&item_1_key)
                .unwrap(),
            &55u32
        );


        assert_eq!(
            backend.datastore.balance_cost_per_user.get(&user_1_key).is_none(),
            true
        );


        //create a bill
        backend.create_bill(0, 100, AllUsers, "remark of bill".to_string());


        assert_eq!(
            backend.datastore.bills[0].bill_state.is_finalized(),
            false
        );
        assert_eq!(
            backend.datastore.purchases.len(),
            12
        );

        backend.update_user(0, "user a".to_string(), true, false, Some("user_id_external_a".to_string()));
        backend.update_user(1, "user b".to_string(), false, false, None);
        backend.update_user(2, "updated_donated_to_user".to_string(), true, false, Some("user_2_external_id".to_string()));


        assert_eq!(
            backend.datastore.open_freebies.get(&2).unwrap().len(),
            2
        );

        assert_eq!(backend.apply(&BLEvents::FinalizeBill{timestamp_from: 0, timestamp_to: 100}) , false);

        assert_eq!(backend.apply(&SetPriceForSpecial {
            unique_id: 10,
            price: 15,
        }), true);

        assert_eq!(backend.apply(&BLEvents::FinalizeBill{timestamp_from: 0, timestamp_to: 100}) , true);


        assert_eq!(
            backend.datastore.bills[0].bill_state.is_finalized(),
            true
        );
        assert_eq!(
            backend.datastore.purchases.len(),
            0
        );
        assert_eq!(
            backend.datastore.bills[0].finalized_data.all_items.len(),
            2
        );
        assert_eq!(
            backend.datastore.bills[0].finalized_data.all_users.len(),
            2
        );
        assert_eq!(
            backend.datastore.bills[0].finalized_data.user_consumption.get(&0).unwrap().per_day.get(&0).unwrap().personally_consumed.len(),
            2
        );
        assert_eq!(
            backend.datastore.bills[0].finalized_data.user_consumption.get(&0).unwrap().per_day.get(&0).unwrap().ffa_giveouts.len(),
            1
        );
        assert_eq!(
            backend.datastore.bills[0].finalized_data.user_consumption.get(&0).unwrap().per_day.get(&0).unwrap().giveouts_to_user_id.len(),
            1
        );
        assert_eq!(
            backend.datastore.bills[0].finalized_data.user_consumption.get(&0).unwrap().per_day.get(&0).unwrap().giveouts_to_user_id.get(&2).unwrap().budget_given,
            190
        );
        assert_eq!(
            backend.datastore.bills[0].finalized_data.user_consumption.get(&2).unwrap().per_day.get(&0).unwrap().giveouts_to_user_id.get(&0).unwrap().budget_gotten,
            190
        );
        assert_eq!(
            backend.datastore.bills[0].finalized_data.user_consumption.get(&2).unwrap().per_day.get(&0).unwrap().giveouts_to_user_id.get(&0).unwrap().budget_given,
            0
        );
        assert_eq!(
            backend.datastore.bills[0].finalized_data.user_consumption.get(&0).unwrap().per_day.get(&0).unwrap().giveouts_to_user_id.get(&2).unwrap().budget_gotten,
            0
        );
        assert_eq!(
            backend.datastore.bills[0].finalized_data.user_consumption.get(&0).unwrap().per_day.get(&0).unwrap().giveouts_to_user_id.get(&2).unwrap().count_giveouts_used.len(),
            2
        );
        assert_eq!(
            backend.datastore.bills[0].finalized_data.user_consumption.get(&0).unwrap().per_day.get(&0).unwrap().specials_consumed.len(),
            1
        );


        assert_eq!(
            backend.datastore.open_freebies.get(&2).unwrap().len(),
            1
        );



        //control that current balance is down to zero for all users
//
//        assert_eq!(
//            backend.datastore.balance_cost_per_user.get(&user_key).is_none(),
//            true
//        );
//
//
//        assert_eq!(
//            backend.datastore.balance_cost_per_user.get(&user_1_key).is_none(),
//            true
//        );


        //control that bill contains correct data
//        assert_eq!(
//            backend
//                .datastore
//                .bills
//                .get(0)
//                .unwrap()
//                .sum_of_cost_hash_map
//                .get(&user_key)
//                .unwrap()
//                .get(&item_0_key)
//                .unwrap(),
//            &90u32
//        );
//        assert_eq!(
//            backend
//                .datastore
//                .bills
//                .get(0)
//                .unwrap()
//                .sum_of_cost_hash_map
//                .get(&user_1_key)
//                .unwrap()
//                .is_empty(),
//            true
//        );
//        assert_eq!(
//            backend
//                .datastore
//                .bills
//                .get(0)
//                .unwrap()
//                .count_hash_map
//                .get(&user_key)
//                .unwrap()
//                .get(&item_1_key)
//                .unwrap(),
//            &1u32
//        );
//        assert_eq!(
//            backend
//                .datastore
//                .bills
//                .get(0)
//                .unwrap()
//                .count_hash_map
//                .get(&user_key)
//                .unwrap()
//                .get(&item_1_key)
//                .unwrap(),
//            &1u32
//        );
//        assert_eq!(
//            backend.datastore.bills.get(0).unwrap().timestamp,
//            100i64
//        );
//        assert_eq!(backend.datastore.bills.get(0).unwrap().users, AllUsers);
//        assert_eq!(
//            backend.datastore.bills.get(0).unwrap().comment,
//            "remark of bill".to_string()
//        );


        //add another purchase and assert that bill didn't change
        backend.purchase(0, 0, 110);
        backend.purchase(1, 2, 120);
        backend.purchase(1, 0, 130);

//        assert_eq!(
//            backend
//                .datastore
//                .bills
//                .get(0)
//                .unwrap()
//                .sum_of_cost_hash_map
//                .get(&user_key)
//                .unwrap()
//                .get(&item_0_key)
//                .unwrap(),
//            &90u32
//        );
//        assert_eq!(
//            backend
//                .datastore
//                .bills
//                .get(0)
//                .unwrap()
//                .sum_of_cost_hash_map
//                .get(&user_1_key)
//                .unwrap()
//                .is_empty(),
//            true
//        );
//        assert_eq!(
//            backend
//                .datastore
//                .bills
//                .get(0)
//                .unwrap()
//                .count_hash_map
//                .get(&user_key)
//                .unwrap()
//                .get(&item_1_key)
//                .unwrap(),
//            &1u32
//        );
//        assert_eq!(
//            backend
//                .datastore
//                .bills
//                .get(0)
//                .unwrap()
//                .count_hash_map
//                .get(&user_key)
//                .unwrap()
//                .get(&item_1_key)
//                .unwrap(),
//            &1u32
//        );
//        assert_eq!(
//            backend.datastore.bills.get(0).unwrap().timestamp,
//            100i64
//        );
//        assert_eq!(backend.datastore.bills.get(0).unwrap().users, AllUsers);
//        assert_eq!(
//            backend.datastore.bills.get(0).unwrap().comment,
//            "remark of bill".to_string()
//        );
    }





    #[test]
    fn simple_ffa_purchase() {
        let mut backend = build_test_backend();
        let ts1 = 1i64;
        let ts2 = 2i64;

        backend.create_user("klaus".to_string());
        backend.create_item("item 1".to_string(), 45, None);
        backend.create_ffa(Vec::new(), vec![0], 1, "some textmessage".to_string(), ts1, 0);

        println!("open_ffa = {:?}\nused_up = {:?}", backend.datastore.open_ffa, backend.datastore.used_up_freebies);
        //freeby should be on new vec
        assert_eq!(backend.datastore.open_ffa.len(), 1);
        assert_eq!(backend.datastore.open_freebies.len(), 0);
        assert_eq!(backend.datastore.used_up_freebies.len(), 0);
        assert_eq!(backend.datastore.open_ffa[0].get_id(), 1);
        backend.ffa_purchase(1, 0, ts2);

        assert_eq!(backend.datastore.purchases.len(), 1);
        assert_eq!(backend.datastore.purchase_count, 1);

        println!("open_ffa = {:?}\nused_up = {:?}", backend.datastore.open_ffa, backend.datastore.used_up_freebies);
        //freeby should be on used up stack and not the new one
        assert_eq!(backend.datastore.open_ffa.len(), 0);
        assert_eq!(backend.datastore.open_freebies.len(), 0);
        assert_eq!(backend.datastore.used_up_freebies.len(), 1);


    }


}
