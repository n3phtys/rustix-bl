// An attribute to hide warnings for unused code.
#![allow(dead_code)]


use std::collections::HashSet;
use std::collections::HashMap;
use left_threaded_avl_tree::ScoredIdTreeMock;
//TODO: finish declaring datastore attributes and functions (mainly getters!)


#[derive(Debug)]
pub struct Datastore {
    pub users: HashMap<u32, User>,
    pub items: HashMap<u32, Item>,
    pub purchases: Vec<Purchase>,
    pub top_user_scores: ScoredIdTreeMock,
    pub top_users: HashSet<u32>,
    pub top_drinks_per_user: HashMap<u32, HashSet<u32>>,
    pub drink_scores_per_user: HashMap<u32, ScoredIdTreeMock>,

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


impl Default for Datastore {
    fn default() -> Self {
        return Datastore {
            users: HashMap::new(),
            items: HashMap::new(),
            purchases: Vec::new(),
            top_user_scores: ScoredIdTreeMock::default(),
            top_users: HashSet::new(),
            top_drinks_per_user: HashMap::new(),
            drink_scores_per_user: HashMap::new(),
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


#[derive(Default, Builder, Debug, PartialEq)]
#[builder(setter(into))]
pub struct User {
    pub username: String,
    //external_user_id: u32, //TODO: external_user_id used in external mapping
    pub user_id: u32,
    //subuser_to: Option<u32>, //TODO: implement to group users
    pub is_billed: bool,
}



#[derive(Default, Builder, Debug)]
#[builder(setter(into))]
pub struct Item {
    pub name: String,
    pub item_id: u32,
    pub category: Option<String>,
    pub cost_cents: u32,
}


#[derive(Default, Builder, Debug)]
#[builder(setter(into))]
pub struct Bill {
    pub timestamp_seconds: u32,
    pub users: UserGroup,
    pub comment: String,
}



#[derive(Debug)]
pub enum Purchase {
    SpecialPurchase {
        timestamp_seconds: u32,
        name: String,
        consumer_id: u32,
    },
    SimplePurchase {
        timestamp_seconds: u32,
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


pub fn test() {

    println!("itemstorage functions:");

    let x = UserBuilder::default()
        //.external_user_id(19124u32)
        .user_id(1234u32)
        //.subuser_to(None)
        .is_billed(true)
        .username("klaus")
        .build();
    println!("{:?}", x);

    let y = ItemBuilder::default()
        .name("cool item")
        .category(None)
        .cost_cents(13u32)
        .item_id(19124u32)
        .build();
    println!("{:?}", y);
}
