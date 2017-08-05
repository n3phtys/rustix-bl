// An attribute to hide warnings for unused code.
#![allow(dead_code)]
#![allow(unused_parens)]
#![allow(unused_variables)]

use datastore::Datastore;
use datastore::UserGroup;
use datastore::Itemable;
use datastore::Userable;

use config::StaticConfig;

use left_threaded_avl_tree::AVLTree;

use std::collections::HashSet;
use std::iter::FromIterator;
use serde_json;
use std;
use serde_json::Error;
use datastore;


pub trait Event {
    fn can_be_applied(&self, store: &Datastore) -> bool;
    fn apply(&self, store: &mut Datastore, config: &StaticConfig) -> ();
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum BLEvents {
    CreateItem {
        itemname: String,
        price_cents: u32,
        category: Option<String>,
    },
    CreateUser { username: String },
    DeleteItem { item_id: u32 },
    DeleteUser { user_id: u32 },
    MakeSimplePurchase {
        user_id: u32,
        item_id: u32,
        timestamp: u32,
    },
    CreateBill {
        timestamp: u32,
        user_ids: UserGroup,
        comment: String,
    },
}

fn hashset(data: &[u32]) -> HashSet<u32> {
    HashSet::from_iter(data.iter().cloned())
}

impl Event for BLEvents {
    fn can_be_applied(&self, store: &Datastore) -> bool {
        return match self {
            &BLEvents::CreateItem {
                ref itemname,
                price_cents,
                ref category,
            } => true,
            &BLEvents::CreateUser { ref username } => true,
            &BLEvents::CreateBill {
                timestamp,
                ref user_ids,
                ref comment,
            } => !store.purchases.is_empty(),
            &BLEvents::DeleteItem { item_id } => store.has_item(item_id),
            &BLEvents::DeleteUser { user_id } => store.has_user(user_id),
            &BLEvents::MakeSimplePurchase {
                user_id,
                item_id,
                timestamp,
            } => store.has_item(item_id) && store.has_user(user_id),
        };
    }

    fn apply(&self, store: &mut Datastore, config: &StaticConfig) -> () {
        return match self {
            &BLEvents::CreateItem {
                ref itemname,
                price_cents,
                ref category,
            } => {
                let id = store.item_id_counter;
                for cat in category.iter() {
                    store.categories.insert(cat.to_string());
                }
                store.items.insert(
                    id,
                    datastore::Item {
                        name: itemname.to_string(),
                        item_id: id,
                        cost_cents: price_cents,
                        category: category.clone(),
                    },
                );
                store.item_id_counter = id + 1u32;
            }
            &BLEvents::CreateUser { ref username } => {
                let id = store.user_id_counter;
                store.users.insert(
                    id,
                    datastore::User {
                        username: username.to_string(),
                        user_id: id,
                        is_billed: true,
                    },
                );
                store.user_id_counter = id + 1u32;
            }
            &BLEvents::CreateBill {
                timestamp,
                ref user_ids,
                ref comment,
            } => unimplemented!(),//TODO:
            &BLEvents::DeleteItem { item_id } => {
                let v = store.items.remove(&item_id);
                match v {
                    None => (),
                    Some(item) => {
                        //potentially remove category, if no one else is sharing that category
                        match item.category {
                            None => (),
                            Some(category) => {
                                if !store.categories.iter().any(|x| x.eq(&category)) {
                                    let _ = store.categories.remove(&category);
                                }
                            }
                        }
                        //remove from personal drink scores and possibly reextract top drinks
                        for (_key, mut value) in &mut store.drink_scores_per_user {
                            value.remove(item_id);
                        }

                        for (_key, value) in &mut store.top_drinks_per_user {
                            match store.drink_scores_per_user.get(&_key) {
                                None => (),
                                Some(drinkscore) => if value.contains(&item_id) {
                                    *value = hashset(
                                        drinkscore
                                            .extract_top(config.top_drinks_per_user as usize)
                                            .as_slice(),
                                    );
                                },
                            }
                        }
                    }
                }
            }
            &BLEvents::DeleteUser { user_id } => {
                //remove from user hashmap
                let _ = store.users.remove(&user_id);

                //remove user item score
                let _ = store.drink_scores_per_user.remove(&user_id);

                //remove user top items
                let _ = store.top_drinks_per_user.remove(&user_id);

                //remove from user score tree
                let _ = store.top_user_scores.remove(user_id);

                //remove from top users and renew topusers if that is the case
                if store.top_users.remove(&user_id) {
                    store.top_users = hashset(
                        store
                            .top_user_scores
                            .extract_top(config.users_in_top_users as usize)
                            .as_slice(),
                    );
                }
            }
            &BLEvents::MakeSimplePurchase {
                user_id,
                item_id,
                timestamp,
            } => unimplemented!(),//TODO:
        };
    }
}

//TODO: finish declaring all possible events here


#[cfg(test)]
mod tests {
    use rustix_event_shop::BLEvents;
    use serde_json;
    use std;


    #[test]
    fn hashset_test() {
        let mut hashset = std::collections::HashSet::new();

        let str_1 = "Hello World".to_string();

        hashset.insert(str_1);

        let str_2_a = "Hello".to_string();
        let str_2_b = " World".to_string();
        let str_2 = str_2_a + &str_2_b;

        assert!(hashset.remove(&str_2))
    }


    #[test]
    fn events_serialize_and_deserialize_raw() {
        let v = vec![
            BLEvents::CreateItem {
                itemname: "beer".to_string(),
                price_cents: 95u32,
                category: None,
            },
            BLEvents::CreateItem {
                itemname: "beer 2".to_string(),
                price_cents: 95u32,
                category: None,
            },
            BLEvents::DeleteItem { item_id: 2u32 },
            BLEvents::CreateUser {
                username: "klaus".to_string(),
            },
            BLEvents::MakeSimplePurchase {
                item_id: 1u32,
                user_id: 1u32,
                timestamp: 123456789u32,
            },
        ];

        // Serialize it to a JSON string.
        let json = serde_json::to_string(&v).unwrap();
        println!("{}", json);
        let reparsed_content: Vec<BLEvents> = serde_json::from_str(&json).unwrap();
        println!("{:#?}", reparsed_content);
        assert_eq!(reparsed_content, v);
    }

    #[test]
    fn events_serialize_and_deserialize_packed() {
        let v = vec![
            BLEvents::CreateItem {
                itemname: "beer".to_string(),
                price_cents: 95u32,
                category: None,
            },
            BLEvents::CreateItem {
                itemname: "beer 2".to_string(),
                price_cents: 95u32,
                category: None,
            },
            BLEvents::DeleteItem { item_id: 2u32 },
            BLEvents::CreateUser {
                username: "klaus".to_string(),
            },
            BLEvents::MakeSimplePurchase {
                item_id: 1u32,
                user_id: 1u32,
                timestamp: 123456789u32,
            },
        ];

        // Serialize it to a JSON string.
        let json_bytes = serde_json::to_string(&v).unwrap().as_bytes().to_vec();
        println!("{:?}", json_bytes);
        let reparsed_content: Vec<BLEvents> =
            serde_json::from_str(std::str::from_utf8(json_bytes.as_ref()).unwrap()).unwrap();
        println!("{:#?}", reparsed_content);
        assert_eq!(reparsed_content, v);
    }
}
