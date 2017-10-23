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
use left_threaded_avl_tree::ScoredIdTreeMock;

use std::collections::HashSet;
use std::collections::HashMap;
use std::iter::FromIterator;
use serde_json;
use std;
use serde_json::Error;
use datastore;
use suffix::*;


pub trait Event {
    fn can_be_applied(&self, store: &Datastore) -> bool;
    fn apply(&self, store: &mut Datastore, config: &StaticConfig) -> bool;
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
        timestamp: i64,
    },
    UndoPurchase {
        unique_id: u64,
    },
    CreateBill {
        timestamp: u32,
        user_ids: UserGroup,
        comment: String,
    },
}

fn hashset(data: &[u32]) -> HashSet<u32> {
    let r = HashSet::from_iter(data.iter().cloned());
    println!("Hashset generated from Vec={:?} to Hashset={:?}", data, r);
    return r;
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
            &BLEvents::UndoPurchase {
                unique_id,
            } => store.purchase_count >= unique_id,
        };
    }

    fn apply(&self, store: &mut Datastore, config: &StaticConfig) -> bool {
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

                for (_, value) in &mut store.drink_scores_per_user {
                    (*value).insert(id);
                }

                true
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

                //add per user scores and top items:
                let mut score_tree = ScoredIdTreeMock::default();
                for (_key, _) in &store.items {
                    let _ = score_tree.insert(*_key);
                }

                store.drink_scores_per_user.insert(id, score_tree);
                store.top_drinks_per_user.insert(id, HashSet::new());

                //add to user scores and reextract:
                store.top_user_scores.insert(id);
                store.top_users = hashset(
                    store
                        .top_user_scores
                        .extract_top(config.users_in_top_users)
                        .as_slice(),
                );

                {
                    let mut users_vec: Vec<datastore::User> = vec![];

                    for (_, v) in &store.users {
                        users_vec.push(v.clone());
                    }

                    store.users_suffix_tree = MockKDTree::build(&users_vec, false);
                }

                true
            }
            &BLEvents::CreateBill {
                timestamp,
                ref user_ids,
                ref comment,
            } => {
                let user_ids_copy: datastore::UserGroup = user_ids.clone();
                let user_ids_other_copy: datastore::UserGroup = user_ids.clone();

                let mut counts: HashMap<u32, HashMap<u32, u32>> = HashMap::new();
                let mut costs: HashMap<u32, HashMap<u32, u32>> = HashMap::new();

                match user_ids_copy {
                    datastore::UserGroup::AllUsers => for user_id in store.users.keys() {
                        counts.insert(
                            *user_id,
                            store
                                .balance_count_per_user
                                .remove(user_id)
                                .unwrap_or(HashMap::new()),
                        );
                        costs.insert(
                            *user_id,
                            store
                                .balance_cost_per_user
                                .remove(user_id)
                                .unwrap_or(HashMap::new()),
                        );
                    },
                    datastore::UserGroup::SingleUser { user_id } => {
                        counts.insert(
                            user_id,
                            store
                                .balance_count_per_user
                                .remove(&user_id)
                                .unwrap_or(HashMap::new()),
                        );
                        costs.insert(
                            user_id,
                            store
                                .balance_cost_per_user
                                .remove(&user_id)
                                .unwrap_or(HashMap::new()),
                        );
                    }
                    datastore::UserGroup::MultipleUsers { ref user_ids } => for user_id in user_ids
                    {
                        counts.insert(
                            *user_id,
                            store
                                .balance_count_per_user
                                .remove(&user_id)
                                .unwrap_or(HashMap::new()),
                        );
                        costs.insert(
                            *user_id,
                            store
                                .balance_cost_per_user
                                .remove(&user_id)
                                .unwrap_or(HashMap::new()),
                        );
                    },
                };

                store.bills.push(datastore::Bill {
                    timestamp_seconds: timestamp,
                    users: user_ids_other_copy,
                    count_hash_map: counts,
                    sum_of_cost_hash_map: costs,
                    comment: comment.to_string(),
                });
                true
            }
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
                true
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



                {
                    let mut users_vec: Vec<datastore::User> = vec![];

                    for (_, v) in &store.users {
                        users_vec.push(v.clone());
                    }

                    store.users_suffix_tree = MockKDTree::build(&users_vec, false);
                }

                //remove from top users and renew topusers if that is the case
                if store.top_users.remove(&user_id) {
                    store.top_users = hashset(
                        store
                            .top_user_scores
                            .extract_top(config.users_in_top_users as usize)
                            .as_slice(),
                    );
                    true
                } else {
                    false
                }
            }
            &BLEvents::MakeSimplePurchase {
                user_id,
                item_id,
                timestamp,
            } => {

                let idx : u64 = store.purchase_count + 1;
                store.purchase_count = idx;

                // add purchase to vector
                store.purchases.push(datastore::Purchase::SimplePurchase {
                    unique_id: idx,
                    timestamp_epoch_millis: timestamp,
                    item_id: item_id,
                    consumer_id: user_id,
                });

                let was_in_before = store.top_users.contains(&user_id);

                // increase item score for user
                if let Some(ref mut drinkscore) = store.drink_scores_per_user.get_mut(&user_id) {
                    drinkscore.increment_by_one(item_id);
                    // if not in top items, potentially extract new set
                    if let Some(topitems) = store.top_drinks_per_user.get_mut(&user_id) {
                        if !(topitems.contains(&item_id)) {
                            *topitems = hashset(
                                drinkscore
                                    .extract_top(config.top_drinks_per_user)
                                    .as_slice(),
                            );
                            println!("New Topitems: {:?}", topitems);
                        }
                    }
                }

                // increase user score
                store.top_user_scores.increment_by_one(user_id);

                // if not in top users, potentially extract new set
                if !(store.top_users.contains(&user_id)) {
                    println!("not in top users for userid = {}", user_id);
                    store.top_users = hashset(
                        store
                            .top_user_scores
                            .extract_top(config.users_in_top_users)
                            .as_slice(),
                    );
                } else {
                    println!("already in top users for userid = {}", user_id);
                }
                println!("New User Scores: {:?}", store.top_user_scores);
                println!("New Top Users: {:?}", store.top_users);

                //increase cost map value
                let alt_hashmap_1 = HashMap::new();
                let mut old_cost_map = store
                    .balance_cost_per_user
                    .remove(&user_id)
                    .unwrap_or(alt_hashmap_1);
                let old_cost_value = *old_cost_map.get(&item_id).unwrap_or(&0);
                old_cost_map.insert(
                    item_id,
                    old_cost_value
                        + store
                            .items
                            .get(&item_id)
                            .map(|item| item.cost_cents)
                            .unwrap_or(0),
                );
                store.balance_cost_per_user.insert(user_id, old_cost_map);

                //increase count map value
                let alt_hashmap_2 = HashMap::new();
                let mut old_count_map = store
                    .balance_count_per_user
                    .remove(&user_id)
                    .unwrap_or(alt_hashmap_2);
                let old_count_value = *old_count_map.get(&item_id).unwrap_or(&0);
                old_count_map.insert(item_id, old_count_value + 1);
                store.balance_count_per_user.insert(user_id, old_count_map);


                let is_in_now = store.top_users.contains(&user_id);

                ((!was_in_before) & (is_in_now))
            },
            &BLEvents::UndoPurchase {
                unique_id,
            } => unimplemented!(),
        };
    }
}


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
                timestamp: 123456789i64,
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
                timestamp: 123456789i64,
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
