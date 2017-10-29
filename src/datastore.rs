// An attribute to hide warnings for unused code.
#![allow(dead_code)]


use std::collections::HashSet;
use std::collections::HashMap;
use left_threaded_avl_tree::ScoredIdTreeMock;
use suffix_rs::*;


#[derive(Debug, Serialize, Deserialize)]
pub struct Datastore {
    pub users: HashMap<u32, User>,
    pub users_suffix_tree: MockKDTree,
    pub items: HashMap<u32, Item>,
    pub purchases: Vec<Purchase>,
    pub purchase_count: u64,
    pub bills: Vec<Bill>,
    pub top_user_scores: ScoredIdTreeMock,
    pub top_users: HashSet<u32>,
    pub top_drinks_per_user: HashMap<u32, HashSet<u32>>,
    pub drink_scores_per_user: HashMap<u32, ScoredIdTreeMock>,
    pub balance_cost_per_user: HashMap<u32, HashMap<u32, u32>>,
    pub balance_count_per_user: HashMap<u32, HashMap<u32, u32>>,

    // keeps hashmap of user_id => user
    // keeps hashmap of user_id => user
    // keeps bill-vector
    // keeps scoring tree for saufbubbies / all users
    // keeps saufbubbies
    // keeps paginated user pages
    // keeps categorized item pages
    // keeps per user item scoring tree
    // keeps per user item simplified bill (hashmap<name,hasmap<price,number>>)
    pub user_id_counter: u32,

    pub item_id_counter: u32,
    pub categories: HashSet<String>,
}

pub trait Itemable {
    fn has_item(&self, id: u32) -> bool;
}

pub trait Userable {
    fn has_user(&self, id: u32) -> bool;
}

pub trait Purchaseable {
    fn get_purchase(&self, id: u64) -> Option<Purchase>;
}

impl Userable for Datastore {
    fn has_user(&self, id: u32) -> bool {
        return self.users.contains_key(&id);
    }
}
impl Itemable for Datastore {
    fn has_item(&self, id: u32) -> bool {
        return self.items.contains_key(&id);
    }
}

impl Purchaseable for Datastore {
    fn get_purchase(&self, id: u64) -> Option<Purchase> {
        for ele in &self.purchases {
            match ele {
                &Purchase::SimplePurchase {
                    ref unique_id,
                    ref timestamp_epoch_millis,
                    ref item_id,
                    ref consumer_id,
                } => {
                    return Some(Purchase::SimplePurchase {
                        unique_id: *unique_id,
                        timestamp_epoch_millis: *timestamp_epoch_millis,
                        item_id: *item_id,
                        consumer_id: *consumer_id,
                    })
                }
                _ => {}
            }
        }
        return None;
    }
}


impl SearchableElement for User {
    fn as_searchable_text(&self) -> String {
        let s: String = self.username.to_string();
        return s;
    }

    fn get_id(&self) -> u32 {
        return self.user_id;
    }
}

impl Default for Datastore {
    fn default() -> Self {
        let empty_user_vec: Vec<User> = Vec::new();

        return Datastore {
            users: HashMap::new(),
            users_suffix_tree: MockKDTree::build(&empty_user_vec, true),
            items: HashMap::new(),
            purchases: Vec::new(),
            purchase_count: 0,
            bills: Vec::new(),
            top_user_scores: ScoredIdTreeMock::default(),
            top_users: HashSet::new(),
            top_drinks_per_user: HashMap::new(),
            drink_scores_per_user: HashMap::new(),
            balance_cost_per_user: HashMap::new(),
            balance_count_per_user: HashMap::new(),
            user_id_counter: 0,
            item_id_counter: 0,
            categories: HashSet::new(),
        };
    }
}



#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum UserGroup {
    SingleUser { user_id: u32 },
    AllUsers,
    MultipleUsers { user_ids: Vec<u32> },
}

impl Default for UserGroup {
    fn default() -> Self {
        return UserGroup::AllUsers;
    }
}


#[derive(Default, Builder, Debug, Serialize, Deserialize, PartialEq)]
#[builder(setter(into))]
pub struct User {
    pub username: String,
    //external_user_id: u32, //TODO: external_user_id used in external mapping
    pub user_id: u32,
    //subuser_to: Option<u32>, //TODO: implement to group users
    pub is_billed: bool,


    //pub cents_since_last_bill: u64,
    //pub cents_since_creation: u64,
}

impl Clone for User {
    fn clone(&self) -> Self {
        return User {
            username: self.username.to_string(),
            user_id: self.user_id,
            is_billed: self.is_billed,
        };
    }
}



#[derive(Default, Builder, Debug, Serialize, Deserialize)]
#[builder(setter(into))]
pub struct Item {
    pub name: String,
    pub item_id: u32,
    pub category: Option<String>,
    pub cost_cents: u32,
}


#[derive(Default, Builder, Debug, Serialize, Deserialize)]
#[builder(setter(into))]
pub struct Bill {
    pub timestamp_seconds: u32,
    pub users: UserGroup,
    pub count_hash_map: HashMap<u32, HashMap<u32, u32>>,
    pub sum_of_cost_hash_map: HashMap<u32, HashMap<u32, u32>>,
    pub comment: String,
}


pub trait PurchaseFunctions {
    fn get_unique_id(&self) -> u64;
    fn has_unique_id(&self, other: u64) -> bool;
    fn get_user_id(&self) -> &u32;
    fn get_item_id(&self) -> &u32;
}


#[derive(Debug, Serialize, Deserialize)]
pub enum Purchase {
    /* SpecialPurchase {
        timestamp_seconds: u32,
        name: String,
        consumer_id: u32,
    },*/
    UndoPurchase { unique_id: u64 },
    SimplePurchase {
        unique_id: u64,
        timestamp_epoch_millis: i64,
        item_id: u32, //buys one instance of this item
        consumer_id: u32,
    }, /*,
    PaidForPurchase {
        timestamp_seconds: u32,
        item_id: u32, //buys one instance of this item
        consumer_id: u32,
        payer_id: u32, //paid for by this person
    }*/
}


impl PurchaseFunctions for Purchase {
    fn get_unique_id(&self) -> u64 {
        match self {
            &Purchase::UndoPurchase { ref unique_id } => {
                return *unique_id;
            }
            &Purchase::SimplePurchase {
                ref unique_id,
                ref timestamp_epoch_millis,
                ref item_id,
                ref consumer_id,
            } => {
                return *unique_id;
            }
        }
    }

    fn has_unique_id(&self, other: u64) -> bool {
        match self {
            &Purchase::UndoPurchase { ref unique_id } => {
                return *unique_id == other;
            }
            &Purchase::SimplePurchase {
                ref unique_id,
                ref timestamp_epoch_millis,
                ref item_id,
                ref consumer_id,
            } => {
                return *unique_id == other;
            }
        }
    }
    fn get_user_id(&self) -> &u32 {
        match self {
            &Purchase::UndoPurchase { ref unique_id } => {
                unimplemented!();
            }
            &Purchase::SimplePurchase {
                ref unique_id,
                ref timestamp_epoch_millis,
                ref item_id,
                ref consumer_id,
            } => {
                return consumer_id;
            }
        }
    }

    fn get_item_id(&self) -> &u32 {
        match self {
            &Purchase::UndoPurchase { ref unique_id } => unimplemented!(),
            &Purchase::SimplePurchase {
                ref unique_id,
                ref timestamp_epoch_millis,
                ref item_id,
                ref consumer_id,
            } => {
                return item_id;
            }
        }
    }
}
