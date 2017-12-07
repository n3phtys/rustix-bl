// An attribute to hide warnings for unused code.
#![allow(dead_code)]


use std::collections::HashSet;
use std::collections::HashMap;
use left_threaded_avl_tree::ScoredIdTreeMock;
use suffix_rs::*;
use suffix_rs::KDTree;
use left_threaded_avl_tree::AVLTree;

pub trait DatastoreQueries {
    fn top_user_ids(&self, n : u16) -> Vec<u32>;
    fn top_item_ids(&self, user_id: u32, n : u8) -> Vec<u32>;

    fn users_searchhit_ids(&self, searchterm: &str) -> Vec<u32>;
    fn items_searchhit_ids(&self, searchterm: &str) -> Vec<u32>;

    fn personal_log_filtered(&self, user_id: u32, millis_start_inclusive: i64, millis_end_exclusive: i64) -> Vec<Purchase>;
    fn global_log_filtered(&self, millis_start_inclusive: i64, millis_end_exclusive: i64) -> &[Purchase];

    fn all_categories(&self) -> Vec<String>;
}


pub trait SuffixTreeRebuildable {
    fn rebuild_user_tree(&self) -> ();
    fn rebuild_item_tree(&self) -> ();
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Datastore {
    pub users: HashMap<u32, User>,
    pub users_suffix_tree: MockKDTree,
    pub items: HashMap<u32, Item>,
    pub items_suffix_tree: MockKDTree,
    pub purchases: Vec<Purchase>,
    pub purchase_count: u64,
    pub bills: Vec<Bill>,
    pub top_user_scores: ScoredIdTreeMock,
    pub top_users: HashSet<u32>,
    pub top_drinks_per_user: HashMap<u32, HashSet<u32>>,
    pub drink_scores_per_user: HashMap<u32, ScoredIdTreeMock>,
    pub balance_cost_per_user: HashMap<(u32, String), HashMap<(u32, String), u32>>,
    pub balance_count_per_user: HashMap<(u32, String), HashMap<(u32, String), u32>>,

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


impl SuffixTreeRebuildable for Datastore {
    fn rebuild_user_tree(&self) -> () {
        unimplemented!()
    }

    fn rebuild_item_tree(&self) -> () {
        unimplemented!()
    }
}


impl DatastoreQueries for Datastore {
    fn top_user_ids(&self, n: u16) -> Vec<u32> {
        return self.top_user_scores.extract_top(n as usize);
    }

    fn users_searchhit_ids(&self, searchterm: &str) -> Vec<u32> {
        let mut v : Vec<u32> = vec![];
        let xs =  self.users_suffix_tree.search(searchterm);
        for x in xs {
            v.push(x.id);
        }
        return v;
    }

    fn items_searchhit_ids(&self, searchterm: &str) -> Vec<u32> {
        return self.items_suffix_tree.search(searchterm).iter().map(|sr : &SearchResult|sr.id).collect();
    }

    fn personal_log_filtered(&self, user_id: u32, millis_start_inclusive: i64, millis_end_exclusive: i64) -> Vec<Purchase> {
        let v : Vec<Purchase> = self.purchases.iter()
            .filter(|p: &&Purchase| {
            p.get_user_id().eq(&user_id) && p.get_timestamp() >= &millis_start_inclusive && p.get_timestamp() < &millis_end_exclusive
        })
            .map(|p: &Purchase| p.clone())
            .collect();

        return v;
    }

    fn global_log_filtered(&self, millis_start_inclusive: i64, millis_end_exclusive: i64) -> &[Purchase] {
        let (from, to) = find_purchase_indices(&self.purchases, millis_start_inclusive, millis_end_exclusive);


        return &self.purchases[from..to];
    }

    fn all_categories(&self) -> Vec<String> {
        let mut v : Vec<String> = Vec::new();
        for x in &self.categories {
            v.push(x.to_string());
        }
        return v;
    }
    fn top_item_ids(&self, user_id: u32, n: u8) -> Vec<u32> {
        match self.drink_scores_per_user.get(&user_id) {
            Some(ref tree) => return tree.extract_top(n as usize),
            None => return vec![],
        };
    }
}

//returns lowest and highest vector index to get all purchases in given timespan
pub fn find_purchase_indices(purchases : &[Purchase], millis_start_inclusive: i64, millis_end_exclusive: i64) -> (usize, usize) {
    let a = purchases.binary_search_by(|probe|probe.get_timestamp().cmp(&millis_start_inclusive));

    let first = match a {
        Ok(b) => b,
        Err(b) => b,
    };

    let o = purchases.binary_search_by(|probe|{
        let c = millis_end_exclusive + 1;
        probe.get_timestamp().cmp(&c)
    });

    let last = match o {
        Ok(g) => g,
        Err(g) => g,
    };

    return (first,last);
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

                    if *unique_id == id {
                        return Some(Purchase::SimplePurchase {
                            unique_id: *unique_id,
                            timestamp_epoch_millis: *timestamp_epoch_millis,
                            item_id: *item_id,
                            consumer_id: *consumer_id,
                        })
                    }
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


impl SearchableElement for Item {
    fn as_searchable_text(&self) -> String {
        let s: String = self.name.to_string();
        return s;
    }

    fn get_id(&self) -> u32 {
        return self.item_id;
    }
}

impl Default for Datastore {
    fn default() -> Self {
        let empty_user_vec: Vec<User> = Vec::new();
        let empty_item_vec: Vec<User> = Vec::new();

        return Datastore {
            users: HashMap::new(),
            users_suffix_tree: MockKDTree::build(&empty_user_vec, true),
            items: HashMap::new(),
            items_suffix_tree: MockKDTree::build(&empty_item_vec, true),
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


#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
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



#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub name: String,
    pub item_id: u32,
    pub category: Option<String>,
    pub cost_cents: u32,
}


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Bill {
    pub timestamp: i64,
    pub users: UserGroup,
    pub count_hash_map: HashMap<(u32, String), HashMap<(u32, String), u32>>,
    pub sum_of_cost_hash_map: HashMap<(u32, String), HashMap<(u32, String), u32>>,
    pub comment: String,
}


pub trait PurchaseFunctions {
    fn get_unique_id(&self) -> u64;
    fn has_unique_id(&self, other: u64) -> bool;
    fn get_user_id(&self) -> &u32;
    fn get_item_id(&self) -> &u32;
    fn get_timestamp(&self) -> &i64;
}





#[derive(Debug, Serialize, Deserialize)]
pub enum Freeby {
    FFA {
        allowed_categories : Vec<String>,
        allowed_drinks : Vec<u32>,
        allowed_number_total : u16,
        allowed_number_used : u16,
        text_message : String,
        created_timestamp : i64,
        donor : u64,
    },
    Transfer {
        cents_worth_total : u64,
        cents_worth_used : u64,
        text_message : String,
        created_timestamp : i64,
        donor : u64,
        recipient : u64,
    },
    Classic {
        allowed_categories : Vec<String>,
        allowed_drinks : Vec<u32>,
        allowed_number_total : u16,
        allowed_number_used : u16,
        text_message : String,
        created_timestamp : i64,
        donor : u64,
        recipient : u64,
    },
}


/*
return match *self {
            Freeby::Classic {
                ref allowed_categories,
                ref allowed_drinks,
                ref allowed_number_total,
                ref allowed_number_used,
                ref text_message,
                ref created_timestamp,
                ref donor,
                ref recipient
            } => {
                uninimplemented!()
            },
            Freeby::Transfer{
                ref cents_worth_total,
                ref cents_worth_used,
                ref text_message,
                ref created_timestamp,
                ref donor,
                ref recipient
            } => {
                uninimplemented!()
            },
            Freeby::FFA {
                ref allowed_categories,
                ref allowed_drinks,
                ref allowed_number_total,
                ref allowed_number_used,
                ref text_message,
                ref created_timestamp,
                ref donor
            } => {
                uninimplemented!()
            },
        };
*/


impl FreebyAble for Freeby {
    fn message(&self) -> &str {
        return match *self {
            Freeby::Classic {
                ref allowed_categories,
                ref allowed_drinks,
                ref allowed_number_total,
                ref allowed_number_used,
                ref text_message,
                ref created_timestamp,
                ref donor,
                ref recipient
            } => {
                text_message
            },
            Freeby::Transfer{
                ref cents_worth_total,
                ref cents_worth_used,
                ref text_message,
                ref created_timestamp,
                ref donor,
                ref recipient
            } => {
                text_message
            },
            Freeby::FFA {
                ref allowed_categories,
                ref allowed_drinks,
                ref allowed_number_total,
                ref allowed_number_used,
                ref text_message,
                ref created_timestamp,
                ref donor
            } => {
                text_message
            },
        };
    }

    fn get_donor(&self) -> u64 {
        return match *self {
            Freeby::Classic {
                ref allowed_categories,
                ref allowed_drinks,
                ref allowed_number_total,
                ref allowed_number_used,
                ref text_message,
                ref created_timestamp,
                ref donor,
                ref recipient
            } => {
                *donor
            },
            Freeby::Transfer{
                ref cents_worth_total,
                ref cents_worth_used,
                ref text_message,
                ref created_timestamp,
                ref donor,
                ref recipient
            } => {
                *donor
            },
            Freeby::FFA {
                ref allowed_categories,
                ref allowed_drinks,
                ref allowed_number_total,
                ref allowed_number_used,
                ref text_message,
                ref created_timestamp,
                ref donor
            } => {
                *donor
            },
        };
    }

    fn allowed_categories(&self) -> &[String] {
        return match *self {
            Freeby::Classic {
                ref allowed_categories,
                ref allowed_drinks,
                ref allowed_number_total,
                ref allowed_number_used,
                ref text_message,
                ref created_timestamp,
                ref donor,
                ref recipient
            } => {
                allowed_categories
            },
            Freeby::Transfer{
                ref cents_worth_total,
                ref cents_worth_used,
                ref text_message,
                ref created_timestamp,
                ref donor,
                ref recipient
            } => {
                &[]
            },
            Freeby::FFA {
                ref allowed_categories,
                ref allowed_drinks,
                ref allowed_number_total,
                ref allowed_number_used,
                ref text_message,
                ref created_timestamp,
                ref donor
            } => {
                allowed_categories
            },
        };
    }

    fn allowed_items(&self) -> &[u32] {
        return match *self {
            Freeby::Classic {
                ref allowed_categories,
                ref allowed_drinks,
                ref allowed_number_total,
                ref allowed_number_used,
                ref text_message,
                ref created_timestamp,
                ref donor,
                ref recipient
            } => {
                allowed_drinks
            },
            Freeby::Transfer{
                ref cents_worth_total,
                ref cents_worth_used,
                ref text_message,
                ref created_timestamp,
                ref donor,
                ref recipient
            } => {
                &[]
            },
            Freeby::FFA {
                ref allowed_categories,
                ref allowed_drinks,
                ref allowed_number_total,
                ref allowed_number_used,
                ref text_message,
                ref created_timestamp,
                ref donor
            } => {
                allowed_drinks
            },
        };
    }

    fn left(&self) -> u16 {
        return match *self {
            Freeby::Classic {
                ref allowed_categories,
                ref allowed_drinks,
                ref allowed_number_total,
                ref allowed_number_used,
                ref text_message,
                ref created_timestamp,
                ref donor,
                ref recipient
            } => {
                (*allowed_number_total) -  (*allowed_number_used)
            },
            Freeby::Transfer{
                ref cents_worth_total,
                ref cents_worth_used,
                ref text_message,
                ref created_timestamp,
                ref donor,
                ref recipient
            } => {
                0u16
            },
            Freeby::FFA {
                ref allowed_categories,
                ref allowed_drinks,
                ref allowed_number_total,
                ref allowed_number_used,
                ref text_message,
                ref created_timestamp,
                ref donor
            } => {
                (*allowed_number_total) -  (*allowed_number_used)
            },
        };
    }
}


pub trait FreebyAble {
    fn message(&self) -> &str;
    fn get_donor(&self) -> u64;
    fn allowed_categories(&self) -> &[String];
    fn allowed_items(&self) -> &[u32];
    fn is_open(&self) -> bool {
        return FreebyAble::left(self) != 0;
    }
    fn left(&self) -> u16;
    fn allows(&self, items: &HashMap<u64, Item>, item_to_allow : &Item) -> bool {
        for id in self.allowed_items() {
            if *id == item_to_allow.item_id {
                return true;
            }
        }
        if let Some(ref cat_to) = item_to_allow.category {
            for cat in self.allowed_categories() {
                if cat.eq(cat_to) {
                    return true;
                }
            }
        }
        return false;
    }
}



#[derive(Debug, Serialize, Deserialize, Clone)]
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

    fn get_timestamp(&self) -> &i64 {
        match self {
            &Purchase::UndoPurchase { ref unique_id } => unimplemented!(),
            &Purchase::SimplePurchase {
                ref unique_id,
                ref timestamp_epoch_millis,
                ref item_id,
                ref consumer_id,
            } => {
                return timestamp_epoch_millis;
            }
        }
    }
}
