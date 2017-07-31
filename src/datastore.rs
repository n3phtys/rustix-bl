// An attribute to hide warnings for unused code.
#![allow(dead_code)]


//TODO: finish declaring datastore attributes and functions (mainly getters!)

pub struct Datastore {
    pub users: Vec<User>,
    pub items: Vec<Item>,
        // keeps hashmap of user_id => user
        // keeps hashmap of user_id => user
        // keeps bill-vector
        // keeps scoring tree for saufbubbies / all users
        // keeps saufbubbies
        // keeps paginated user pages
        // keeps categorized item pages
        // keeps per user item scoring tree
        // keeps per user item simplified bill (hashmap<name,hasmap<price,number>>)

        //user_id counter

        //item_id counter
}



#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum UserGroup {
    SingleUser{user_id: u32},
    AllUsers,
    MultipleUsers{user_ids: Vec<u32>},
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
    name: String,
    item_id: u32,
    category_id: Option<u32>,
    cost_cents: u32,
}


#[derive(Default, Builder, Debug)]
#[builder(setter(into))]
pub struct Bill {
    timestamp_seconds: u32,
    users: UserGroup,
    comment: String,
}



#[derive(Debug)]
pub enum Purchase {
    SpecialPurchase { timestamp_seconds: u32, name: String, consumer_id: u32},
    SimplePurchase {
        timestamp_seconds: u32,
        item_id: u32, //buys one instance of this item
        consumer_id: u32,
    }
    /*,
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
        .build()
    ;
    println!("{:?}", x);

    let y = ItemBuilder::default()
        .name("cool item")
        .category_id(None)
        .cost_cents(13u32)
        .item_id(19124u32)
        .build()
    ;
    println!("{:?}", y);
}