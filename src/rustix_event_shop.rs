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
use datastore::*;
use suffix_rs::*;
use datastore::PurchaseFunctions;


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
    UpdateUser { user_id: u32, username: String, is_billed: bool, is_highlighted: bool, external_user_id: Option<String>},
    UpdateItem {
        item_id: u32,
        itemname: String,
        price_cents: u32,
        category: Option<String>,},
    DeleteItem { item_id: u32 },
    DeleteUser { user_id: u32 },
    MakeSimplePurchase {
        user_id: u32,
        item_id: u32,
        timestamp: i64,
    },
    MakeShoppingCartPurchase {
        user_id: u32,
        specials : Vec<String>,
        item_ids : Vec<u32>,
        timestamp: i64,
    },
    MakeSpecialPurchase {
        user_id: u32,
        special_name: String,
        timestamp: i64,
    },
    MakeFreeForAllPurchase {
        ffa_id: u64,
        item_id: u32,
        timestamp: i64,
    },
    CreateFreeForAll {
        allowed_categories : Vec<String>,
        allowed_drinks : Vec<u32>,
        allowed_number_total : u16,
        text_message : String,
        created_timestamp : i64,
        donor : u32,
    },
    CreateFreeCount {
        allowed_categories : Vec<String>,
        allowed_drinks : Vec<u32>,
        allowed_number_total : u16,
        text_message : String,
        created_timestamp : i64,
        donor : u32,
        recipient : u32,
    },
    CreateFreeBudget {
        cents_worth_total : u64,
        text_message : String,
        created_timestamp : i64,
        donor : u32,
        recipient : u32,
    },
    UndoPurchase { unique_id: u64 },
    CreateBill {
        timestamp_from: i64,
        timestamp_to: i64,
        user_ids: UserGroup,
        comment: String,
    },
    FinalizeBill {
        timestamp_from: i64, //timestamps uniquely identify a bill
        timestamp_to: i64,
    },
    ExportBill {
        timestamp_from: i64, //timestamps uniquely identify a bill
        timestamp_to: i64,
    },
    DeleteUnfinishedBill {
        timestamp_from: i64, //timestamps uniquely identify a bill
        timestamp_to: i64,
    },
    SetPriceForSpecial {
        unique_id: u64,
        price: u32,
    }
}

fn hashset(data: &[u32]) -> HashSet<u32> {
    let r = HashSet::from_iter(data.iter().cloned());
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
                ref timestamp_from,
                ref timestamp_to,
                ref user_ids,
                ref comment,
            } => {
                !store.purchases.is_empty() &&
                    !store.bills.iter().any(|b| b.timestamp_to == *timestamp_to && b.timestamp_from == *timestamp_from)
            },
            &BLEvents::UpdateItem { ref item_id,
                ref itemname,
                ref price_cents,
                ref category } => store.has_item(*item_id),
            &BLEvents::UpdateUser { ref user_id, ref username, ref is_billed, ref is_highlighted, ref external_user_id } => store.has_user(*user_id),
            &BLEvents::DeleteItem { item_id } => store.has_item(item_id),
            &BLEvents::DeleteUser { user_id } => store.has_user(user_id),
            &BLEvents::MakeSimplePurchase {
                user_id,
                item_id,
                timestamp,
            } => store.has_item(item_id) && store.has_user(user_id),
            &BLEvents::MakeSpecialPurchase { ref user_id, ref special_name, ref timestamp } => store.has_user(*user_id),
            &BLEvents::MakeShoppingCartPurchase { ref user_id, ref specials, ref item_ids, ref timestamp } => {

            let mut v : Vec<BLEvents> = Vec::new();
            for x in item_ids {
            v.push(BLEvents::MakeSimplePurchase {user_id: *user_id, item_id: *x, timestamp: *timestamp});
            }
            for x in specials {
            v.push(BLEvents::MakeSpecialPurchase {user_id: *user_id, special_name: x.to_string(), timestamp: *timestamp});
            }

            let mut result = true;
            for x in v {
                result = result & x.can_be_applied(store);
            }
            return result;
            }
            &BLEvents::MakeFreeForAllPurchase { ffa_id, item_id, timestamp } => {
                let mut b = false;
                let x : Option<&Freeby> = store.open_ffa.iter().find(|x|x.get_id() == ffa_id);
                let item: &Item = store.items.get(&item_id).unwrap();
                x.filter(|ffa|ffa.allows(item)).is_some()
            },
            &BLEvents::CreateFreeForAll { ref allowed_categories, ref allowed_drinks, ref allowed_number_total, ref text_message, ref created_timestamp, ref donor  } =>
                {
                    return true;
                },
            &BLEvents::CreateFreeCount { ref allowed_categories, ref allowed_drinks, ref allowed_number_total, ref text_message, ref created_timestamp, ref donor, ref recipient } => unimplemented!(),
            &BLEvents::CreateFreeBudget { ref cents_worth_total, ref text_message, ref created_timestamp, ref donor, ref recipient } => unimplemented!(),
            &BLEvents::UndoPurchase { unique_id } => store.get_purchase(unique_id).is_some(),
            &BLEvents::FinalizeBill {  timestamp_from, timestamp_to } => {
                store.get_bill(timestamp_from, timestamp_to).filter(|b|b.bill_state.is_finalized()).is_some()
            },
                &BLEvents::ExportBill {  timestamp_from, timestamp_to } => {
                store.get_un_set_users_to_bill(timestamp_from, timestamp_to).is_empty() && store.get_unpriced_specials_to_bill(timestamp_from, timestamp_to).is_empty()
            },
            &BLEvents::DeleteUnfinishedBill { timestamp_from, timestamp_to } => store.get_bill(timestamp_from, timestamp_to).filter(|b|b.bill_state.is_created()).is_some(),
            &BLEvents::SetPriceForSpecial { unique_id, price } => store.get_purchase(unique_id).is_some(),
        };
    }

    fn apply(&self, store: &mut Datastore, config: &StaticConfig) -> bool {
        return match self {
            &BLEvents::CreateItem {
                ref itemname,
                ref price_cents,
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
                        cost_cents: *price_cents,
                        category: category.clone(),
                        deleted: false,
                    },
                );
                store.item_id_counter = id + 1u32;

                for (_, value) in &mut store.drink_scores_per_user {
                    (*value).insert(id);
                }



                {
                    let mut items_vec: Vec<datastore::Item> = vec![];

                    for (_, v) in &store.items {
                        let copy : datastore::Item = (v.clone());
                        items_vec.push(copy);
                    }

                    store.items_suffix_tree = MockKDTree::build(&items_vec, false);
                }


                true
            }
            &BLEvents::CreateUser { ref username } => {
                let id = store.user_id_counter;
                store.users.insert(
                    id,
                    datastore::User {
                        username: username.to_string(),
                        external_user_id: None,
                        user_id: id,
                        is_billed: true,
                        highlight_in_ui: false,
                        deleted: false,
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
            },
            &BLEvents::UpdateItem { ref item_id,
                ref itemname,
                ref price_cents,
                ref category } => {
                let mut e = store.items.get_mut(item_id).unwrap();
                e.name = itemname.to_string();
                e.cost_cents = *price_cents;
                e.category = category.clone();

                true
            },
            &BLEvents::UpdateUser { ref user_id, ref username, ref is_billed, ref is_highlighted, ref external_user_id } => {
                let mut e = store.users.get_mut(user_id).unwrap();
                e.username = username.to_string();
                e.is_billed = *is_billed;
                e.highlight_in_ui = *is_highlighted;
                e.external_user_id = external_user_id.clone();

                //if highlight_in_ui changed, update store.highlighted_users
                if *is_highlighted {
                    let _ = store.highlighted_users.insert(*user_id);
                } else {
                    let _ = store.highlighted_users.remove(user_id);
                }

                true
            },
            &BLEvents::CreateBill {
                ref timestamp_from,
                ref timestamp_to,
                ref user_ids,
                ref comment,
            } => {
                store.bills.push(datastore::Bill {
                    timestamp_from: *timestamp_from,
                    timestamp_to: *timestamp_to,
                    users: user_ids.clone(),
                    bill_state: datastore::BillState::Created,
                    users_that_will_not_be_billed: HashSet::new(),
                    comment: comment.to_string(),
                    finalized_data: datastore::ExportableBillData {
                        all_users: HashMap::new(),
                        all_items: HashMap::new(),
                        user_consumption: HashMap::new(),
                    },
                });
                true
            }
            &BLEvents::DeleteItem { item_id } => {
                let _ = store.items.get_mut(&item_id).map(|it|{it.deleted = true});
                let v = store.items.get(&item_id);
                match v {
                    None => (),
                    Some(item) => {
                        //potentially remove category, if no one else is sharing that category
                        match item.category {
                            None => (),
                            Some(ref category) => {
                                if !store.categories.iter().any(|x| x.eq(category)) {
                                    let _ = store.categories.remove(&category.clone());
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

                {
                    let mut items_vec: Vec<datastore::Item> = vec![];

                    for (_, v) in &store.items {
                        if !v.deleted {
                            items_vec.push(v.clone());
                        }
                    }

                    store.items_suffix_tree = MockKDTree::build(&items_vec, false);
                }
                true
            }
            &BLEvents::DeleteUser { user_id } => {


                //if highlight_in_ui, update store.highlighted_users
                if store.users.get(&user_id).filter(|x|x.highlight_in_ui).is_some() {
                    let _ = store.highlighted_users.remove(&user_id);
                }

                //remove from user hashmap
                let _ = store.users.get_mut(&user_id).map(|it|it.deleted = true);

                //remove user item score
                let _ = store.drink_scores_per_user.remove(&user_id);

                //remove user top items
                let _ = store.top_drinks_per_user.remove(&user_id);

                //remove from user score tree
                let _ = store.top_user_scores.remove(user_id);



                {
                    let mut users_vec: Vec<datastore::User> = vec![];

                    for (_, v) in &store.users {
                        if !v.deleted {
                            users_vec.push(v.clone());
                        }
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
            //should return true if it was the most recent purchase

            &BLEvents::MakeSimplePurchase {
                user_id,
                item_id,
                timestamp,
            } => {
                let idx: u64 = store.purchase_count + 1;
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

                }

                //increase cost map value
                let alt_hashmap_1 = HashMap::new();
                let username = store.users.get(&user_id).unwrap().username.to_string();
                let itemname = store.items.get(&item_id).unwrap().name.to_string();
                let user_key = (user_id, store.users.get(&user_id).unwrap().username.to_string());
                let item_key = (item_id, store.items.get(&item_id).unwrap().name.to_string());
                let mut old_cost_map = store
                    .balance_cost_per_user
                    .remove(&user_key)
                    .unwrap_or(alt_hashmap_1);
                let old_cost_value = *old_cost_map.get(&item_key).unwrap_or(&0);
                old_cost_map.insert(
                    item_key,
                    old_cost_value
                        + store
                        .items
                        .get(&item_id)
                        .map(|item| item.cost_cents)
                        .unwrap_or(0),
                );
                store.balance_cost_per_user.insert(user_key, old_cost_map);

                //increase count map value
                let alt_hashmap_2 = HashMap::new();
                let user_key2 : (u32, String) = (user_id, username.to_string());
                let user_key3 : (u32, String) = (user_id, username.to_string());
                let item_key2 : (u32, String) = (item_id, itemname.to_string());
                let item_key3 : (u32, String) = (item_id, itemname.to_string());
                let mut old_count_map = store
                    .balance_count_per_user
                    .remove(&user_key2)
                    .unwrap_or(alt_hashmap_2);
                let old_count_value = *old_count_map.get(&item_key2).unwrap_or(&0);
                old_count_map.insert(item_key3, old_count_value + 1);
                store.balance_count_per_user.insert(user_key3, old_count_map);


                let is_in_now = store.top_users.contains(&user_id);



                ((!was_in_before) & (is_in_now))
            }
            &BLEvents::MakeSpecialPurchase { ref user_id, ref special_name, ref timestamp } => {

                let idx: u64 = store.purchase_count + 1;
                store.purchase_count = idx;


                store.purchases.push(datastore::Purchase::SpecialPurchase {
                    unique_id: idx,
                    timestamp_epoch_millis: *timestamp,
                    special_name: special_name.to_string(),
                    specialcost: None,
                    consumer_id: *user_id,
                });

                true
            },
            &BLEvents::MakeShoppingCartPurchase { ref user_id, ref specials, ref item_ids, ref timestamp } => {
                let mut v : Vec<BLEvents> = Vec::new();
                for x in item_ids {
                    v.push(BLEvents::MakeSimplePurchase {user_id: *user_id, item_id: *x, timestamp: *timestamp});
                }
                for x in specials {
                    v.push(BLEvents::MakeSpecialPurchase {user_id: *user_id, special_name: x.to_string(), timestamp: *timestamp});
                }

                let mut result = true;
                for x in v {
                    result = result & x.apply(store, config);
                }
                return result;
            },

            &BLEvents::MakeFreeForAllPurchase { ffa_id, item_id, timestamp } => {

                //get new id
                let idx: u64 = store.purchase_count + 1;
                store.purchase_count = idx;


                let index = store.open_ffa.iter().position(|x| x.get_id() == ffa_id).unwrap();
                let mut freeby : Freeby = store.open_ffa.remove(index);

                let user_id: u32 = freeby.get_donor();

                {
                //add to purchase vector
                store.purchases.push(datastore::Purchase::FFAPurchase {
                    unique_id: idx,
                    timestamp_epoch_millis: timestamp,
                    item_id: item_id,
                    freeby_id: freeby.get_id(),
                    donor: freeby.get_donor(),
                });
            }

                {

                    println!("Freeby was : {:?}", freeby);
                    //decrease existing freeby
                freeby.decrement();
                    println!("Freeby is : {:?}", freeby);
            }

                //potentially move used up freeby to "old" stack
                if freeby.left() == 0 {
                    //add to new vec
                    store.used_up_freebies.push(freeby);
                } else {
                    store.open_ffa.insert(index, freeby);
                }

                //add to cost / count map of donor

                {
                    //increase cost map value
                    let alt_hashmap_1 = HashMap::new();
                    let username = store.users.get(&user_id).unwrap().username.to_string();
                    let itemname = store.items.get(&item_id).unwrap().name.to_string();
                    let user_key = (user_id, store.users.get(&user_id).unwrap().username.to_string());
                    let item_key = (item_id, store.items.get(&item_id).unwrap().name.to_string());
                    let mut old_cost_map = store
                        .balance_cost_per_user
                        .remove(&user_key)
                        .unwrap_or(alt_hashmap_1);
                    let old_cost_value = *old_cost_map.get(&item_key).unwrap_or(&0);
                    old_cost_map.insert(
                        item_key,
                        old_cost_value
                            + store
                            .items
                            .get(&item_id)
                            .map(|item| item.cost_cents)
                            .unwrap_or(0),
                    );
                    store.balance_cost_per_user.insert(user_key, old_cost_map);

                    //increase count map value
                    let alt_hashmap_2 = HashMap::new();
                    let user_key2 : (u32, String) = (user_id, username.to_string());
                    let user_key3 : (u32, String) = (user_id, username.to_string());
                    let item_key2 : (u32, String) = (item_id, itemname.to_string());
                    let item_key3 : (u32, String) = (item_id, itemname.to_string());
                    let mut old_count_map = store
                        .balance_count_per_user
                        .remove(&user_key2)
                        .unwrap_or(alt_hashmap_2);
                    let old_count_value = *old_count_map.get(&item_key2).unwrap_or(&0);
                    old_count_map.insert(item_key3, old_count_value + 1);
                    store.balance_count_per_user.insert(user_key3, old_count_map);
                }

                true

                /*


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

                }

                //increase cost map value
                let alt_hashmap_1 = HashMap::new();
                let username = store.users.get(&user_id).unwrap().username.to_string();
                let itemname = store.items.get(&item_id).unwrap().name.to_string();
                let user_key = (user_id, store.users.get(&user_id).unwrap().username.to_string());
                let item_key = (item_id, store.items.get(&item_id).unwrap().name.to_string());
                let mut old_cost_map = store
                    .balance_cost_per_user
                    .remove(&user_key)
                    .unwrap_or(alt_hashmap_1);
                let old_cost_value = *old_cost_map.get(&item_key).unwrap_or(&0);
                old_cost_map.insert(
                    item_key,
                    old_cost_value
                        + store
                        .items
                        .get(&item_id)
                        .map(|item| item.cost_cents)
                        .unwrap_or(0),
                );
                store.balance_cost_per_user.insert(user_key, old_cost_map);

                //increase count map value
                let alt_hashmap_2 = HashMap::new();
                let user_key2 : (u32, String) = (user_id, username.to_string());
                let user_key3 : (u32, String) = (user_id, username.to_string());
                let item_key2 : (u32, String) = (item_id, itemname.to_string());
                let item_key3 : (u32, String) = (item_id, itemname.to_string());
                let mut old_count_map = store
                    .balance_count_per_user
                    .remove(&user_key2)
                    .unwrap_or(alt_hashmap_2);
                let old_count_value = *old_count_map.get(&item_key2).unwrap_or(&0);
                old_count_map.insert(item_key3, old_count_value + 1);
                store.balance_count_per_user.insert(user_key3, old_count_map);


                let is_in_now = store.top_users.contains(&user_id);

                ((!was_in_before) & (is_in_now))*/
            },
            &BLEvents::CreateFreeForAll { ref allowed_categories, ref allowed_drinks, ref allowed_number_total, ref text_message, ref created_timestamp, ref donor  } => {
                let id = {
                    let x = store.freeby_id_counter + 1;
                    store.freeby_id_counter = x;
                    x
                };

                store.open_ffa.push(Freeby::FFA{
                id: id,
                allowed_categories: allowed_categories.to_vec(),
                allowed_drinks: allowed_drinks.to_vec(),
                allowed_number_total: *allowed_number_total,
                allowed_number_used : 0,
                text_message : text_message.to_string(),
                created_timestamp : *created_timestamp,
                donor: *donor,
                });


                true
            },
            &BLEvents::ExportBill {  timestamp_from, timestamp_to } => {
                let b: &mut Bill = store.get_mut_bill(timestamp_from, timestamp_to).unwrap();
                b.bill_state = datastore::BillState::ExportedAtLeastOnce;
                true
            },
            &BLEvents::CreateFreeCount { ref allowed_categories, ref allowed_drinks, ref allowed_number_total, ref text_message, ref created_timestamp, ref donor, ref recipient } => {
                let id = {
                    let x = store.freeby_id_counter + 1;
                    store.freeby_id_counter = x;
                    x
                };
                if !store.open_freebies.contains_key(recipient) {
                    store.open_freebies.insert(*recipient, vec![]);
                }
                store.open_freebies.get_mut(recipient).unwrap().push( datastore::Freeby::Classic {
                    id: id,
                    allowed_categories: allowed_categories.to_vec(),
                    allowed_drinks: allowed_drinks.to_vec(),
                    allowed_number_total: *allowed_number_total,
                    allowed_number_used: 0,
                    text_message: text_message.to_string(),
                    created_timestamp: *created_timestamp,
                    donor: *donor,
                    recipient: *recipient,
                });

                true
            },
            &BLEvents::CreateFreeBudget { ref cents_worth_total, ref text_message, ref created_timestamp, ref donor, ref recipient } => {
                let id = {
                    let x = store.freeby_id_counter + 1;
                    store.freeby_id_counter = x;
                    x
                };
                if !store.open_freebies.contains_key(recipient) {
                    store.open_freebies.insert(*recipient, vec![]);
                }
                store.open_freebies.get_mut(recipient).unwrap().push( datastore::Freeby::Transfer {
                    id: id,
                    cents_worth_total: *cents_worth_total,
                    cents_worth_used: 0,
                    text_message: text_message.to_string(),
                    created_timestamp: *created_timestamp,
                    donor: *donor,
                    recipient: *recipient,
                });

                true
            },
            &BLEvents::UndoPurchase { unique_id } => {
                //remove purchase from list
                let index = store
                    .purchases
                    .iter()
                    .position(|x| x.has_unique_id(unique_id))
                    .unwrap();
                let old_size = store.purchases.len();
                let element = store.purchases.remove(index);

                let item_key1 : (u32, String) = (*element.get_item_id(), store.items.get(&element.get_item_id()).unwrap().name.to_string());
                let item_key2 : (u32, String) = (*element.get_item_id(), store.items.get(&element.get_item_id()).unwrap().name.to_string());
                let user_key : (u32, String) = (*element.get_user_id(), store.users.get(&element.get_user_id()).unwrap().username.to_string());

                //remove cost and count lists:
                let oldcost = *store
                    .balance_cost_per_user
                    .get(&user_key)
                    .unwrap()
                    .get(&item_key1)
                    .unwrap();
                let oldcount = *store
                    .balance_count_per_user
                    .get(&user_key)
                    .unwrap()
                    .get(&item_key1)
                    .unwrap();


                store
                    .balance_cost_per_user
                    .get_mut(&user_key)
                    .unwrap()
                    .insert(item_key1, oldcost - 1);
                store
                    .balance_count_per_user
                    .get_mut(&user_key)
                    .unwrap()
                    .insert(item_key2, oldcount - 1);


                old_size == index + 1
            },
            //removes purchases from global list and also recomputes counts
            &BLEvents::FinalizeBill {  timestamp_from, timestamp_to } => {

                let bill_idx: usize = store.get_bill_index(timestamp_from, timestamp_to).unwrap();

                let bill_cpy: Bill = store.bills[bill_idx].clone();

                let purchase_indices = store.get_purchase_indices_to_bill(&bill_cpy);

                //TODO: compute users and create copies
                {
                    let filtered_purs : Vec<Purchase> = purchase_indices.iter().map(|idx| store.purchases[*idx].clone()).collect();
                    for purchase in filtered_purs {

                        let user : User = store.users[purchase.get_user_id()].clone();

                        match purchase {
                            Purchase::SpecialPurchase{
                                unique_id,
                                timestamp_epoch_millis,
                                special_name,
                                specialcost,
                                consumer_id,
                            } => {
                                let day_idx : usize = bill_cpy.get_day_index(timestamp_epoch_millis);
                                let mut bill: &mut Bill = store.bills.get_mut(bill_idx).unwrap();
                                if !bill.finalized_data.all_users.contains_key(&consumer_id) {
                                    bill.finalized_data.all_users.insert(consumer_id, user.clone());
                                    bill.finalized_data.user_consumption.insert(consumer_id, BillUserInstance {
                                        user_id: consumer_id,
                                        per_day: HashMap::new(),
                                    });
                                }

                                if !bill.finalized_data.user_consumption.get_mut(&consumer_id).unwrap().per_day.contains_key(&day_idx) {
                                    bill.finalized_data.user_consumption.get_mut(&consumer_id).unwrap().per_day.insert(day_idx, BillUserDayInstance {
                                        personally_consumed: HashMap::new(),
                                        specials_consumed: Vec::new(),
                                        ffa_giveouts: HashMap::new(),
                                        giveouts_to_user_id: HashMap::new(),
                                    });
                                }
                                bill.finalized_data.user_consumption.get_mut(&consumer_id).unwrap().per_day.get_mut(&day_idx).unwrap().specials_consumed.push(PricedSpecial {
                                    purchase_id: unique_id,
                                    price: specialcost.unwrap(),
                                    name: special_name.to_string(),
                                });
                            },
                            Purchase::SimplePurchase  {
                                unique_id,
                                timestamp_epoch_millis,
                                item_id,
                                consumer_id,
                            } => {

                                //TODO: do not forget to use count and budget giveouts here, if they exist
                                unimplemented!()
                            },
                            Purchase::FFAPurchase {
                                unique_id,
                                timestamp_epoch_millis,
                                item_id,
                                freeby_id,
                                donor,
                            } => {
                                let day_idx : usize = bill_cpy.get_day_index(timestamp_epoch_millis);
                                let mut bill: &mut Bill = store.bills.get_mut(bill_idx).unwrap();
                                if !bill.finalized_data.all_users.contains_key(&donor) {
                                    bill.finalized_data.all_users.insert(donor, user.clone());
                                    bill.finalized_data.user_consumption.insert(donor, BillUserInstance {
                                        user_id: donor,
                                        per_day: HashMap::new(),
                                    });
                                }

                                if !bill.finalized_data.user_consumption[&donor].per_day.contains_key(&day_idx) {
                                    bill.finalized_data.user_consumption.get_mut(&donor).unwrap().per_day.insert(day_idx, BillUserDayInstance {
                                        personally_consumed: HashMap::new(),
                                        specials_consumed: Vec::new(),
                                        ffa_giveouts: HashMap::new(),
                                        giveouts_to_user_id: HashMap::new(),
                                    });
                                }

                                *bill.finalized_data.user_consumption.get_mut(&donor).unwrap().per_day.get_mut(&day_idx).unwrap().ffa_giveouts.entry(item_id).or_insert(0u32) += 1;
                            },
                        }

                }
                }


                //TODO: iter() over all purchases in given time, add to maps via mapping function; original purchases are deleted from purchases vec
                //TODO: for every purchase, look at user's open freebies first, and potentially consume one, in which case the purchase will be added to the donor instead
                //TODO: when total > budget > 0, add budget as negative to recipient and as positive to donor, with exact names
                //TODO: if freeby is empty, move to 'old' (keep 'old' ordered by timestamp for binary search)
                //TODO: whenever a new item is found in purchases, add a copy to bill's hashmap of items
                //TODO: add priced specials per day per user
                //TODO: balance_cost_per_user also has to be reduced for each purchase

                //remove purchases from purchases vec
                {
                    store.remove_purchases_indices(purchase_indices);
                }
                true

                //TODO: Open question: how will purchase rank be recomputed? Currently kept
            },
            &BLEvents::DeleteUnfinishedBill { timestamp_from, timestamp_to } => {
                let idx_opt : Option<usize> = store.bills.iter().position(|b|b.timestamp_to == timestamp_to && b.timestamp_from == timestamp_from);
                match idx_opt {
                    Some(idx) => {
                        let _ = store.bills.remove(idx);
                        return true;
                    }
                    None => {
                        return false;
                    },
                }
            },
            &BLEvents::SetPriceForSpecial { unique_id, price } => {
                let x = store.get_purchase_mut(unique_id).unwrap();

                match x {
                    &mut Purchase::SpecialPurchase{
                        ref unique_id,
                        ref timestamp_epoch_millis,
                        ref special_name,
                        ref mut specialcost,
                        ref consumer_id,
                    } => {
                        *specialcost = Some(price);
                        return true;
                    },
                    _ => return false,
                }
            },
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
        let reparsed_content: Vec<BLEvents> = serde_json::from_str(&json).unwrap();
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
        let reparsed_content: Vec<BLEvents> =
            serde_json::from_str(std::str::from_utf8(json_bytes.as_ref()).unwrap()).unwrap();
        assert_eq!(reparsed_content, v);
    }
}
