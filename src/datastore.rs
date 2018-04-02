// An attribute to hide warnings for unused code.
#![allow(dead_code)]


use std::collections::HashSet;
use std::collections::HashMap;
use left_threaded_avl_tree::ScoredIdTreeMock;
use suffix_rs::*;
use suffix_rs::KDTree;
use left_threaded_avl_tree::AVLTree;
use typescriptify::TypeScriptifyTrait;

pub trait DatastoreQueries {
    fn get_purchase_timestamp(&self, purchase_id: u64) -> Option<i64>;
    fn top_user_ids(&self, n : u16) -> Vec<u32>;
    fn top_item_ids(&self, user_id: u32, n : u8) -> Vec<u32>;

    fn users_searchhit_ids(&self, searchterm: &str) -> Vec<u32>;
    fn items_searchhit_ids(&self, searchterm: &str) -> Vec<u32>;

    fn personal_log_filtered(&self, user_id: u32, millis_start_inclusive: i64, millis_end_exclusive: i64) -> Vec<Purchase>;
    fn global_log_filtered(&self, millis_start_inclusive: i64, millis_end_exclusive: i64) -> &[Purchase];

    fn bills_filtered(&self, user_id: Option<u32>, millis_start_inclusive: i64, millis_end_exclusive: i64) -> Vec<Bill>;

    fn all_categories(&self) -> Vec<String>;


    fn get_mut_purchase(&mut self, id: &u64) -> Option<&mut Purchase>;
    fn get_bill(&self, timestamp_from: i64, timestamp_to: i64) -> Option<&Bill>;
    fn get_mut_bill(&mut self, timestamp_from: i64, timestamp_to: i64) -> Option<&mut Bill>;


    fn get_specials_to_bill(&self, timestamp_from: i64, timestamp_to: i64) -> Vec<u64>;

    fn get_unpriced_specials_to_bill(&self, timestamp_from: i64, timestamp_to: i64) -> Vec<u64>;

    //can have external_id, not_billed flag set, or in ignore list. Will still be shown
    fn get_users_to_bill(&self, timestamp_from: i64, timestamp_to: i64) -> Vec<u32>;
    fn get_un_set_users_to_bill(&self, timestamp_from: i64, timestamp_to: i64) -> Vec<u32>;
    fn get_bill_index(&mut self, timestamp_from: i64, timestamp_to: i64) -> Option<usize>;
    fn get_purchase_indices_to_bill(&self, bill: &Bill) -> Vec<usize>;


    fn remove_purchases_indices(&mut self, indices: Vec<usize>);


    fn get_ffa_freeby(&self, id: u64) -> Option<&Freeby>;
    fn get_personal_freeby(&self, recipient_id: u32, freeby_id: u64) -> Option<&Freeby>;




    fn get_budget_freeby_id_useable_for(&self, recipient_id: u32) -> Option<usize>;
    fn get_count_freeby_id_useable_for(&self, recipient_id: u32, item : u32) -> Option<usize>;
}


pub trait SuffixTreeRebuildable {
    fn rebuild_user_tree(&self) -> ();
    fn rebuild_item_tree(&self) -> ();
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Datastore {

    pub version: u64,
    pub user_id_counter: u32,
    pub freeby_id_counter: u64,
    pub item_id_counter: u32,

    pub users: HashMap<u32, User>,
    pub users_suffix_tree: MockKDTree,
    pub items: HashMap<u32, Item>,
    pub items_suffix_tree: MockKDTree,
    pub purchases: Vec<Purchase>,
    pub purchase_count: u64,
    pub bills: Vec<Bill>,
    pub top_user_scores: ScoredIdTreeMock,
    pub top_users: HashSet<u32>,
    pub highlighted_users: HashSet<u32>,
    pub top_drinks_per_user: HashMap<u32, HashSet<u32>>,
    pub drink_scores_per_user: HashMap<u32, ScoredIdTreeMock>,
    pub balance_cost_per_user: HashMap<(u32, String), HashMap<(u32, String), u32>>,
    pub balance_count_per_user: HashMap<(u32, String), HashMap<(u32, String), u32>>,
    pub used_up_freebies: Vec<Freeby>, //completely mixed
    pub open_freebies: HashMap<u32, Vec<Freeby>>, //per recipient
    pub open_ffa: Vec<Freeby>,

    // keeps hashmap of user_id => user
    // keeps hashmap of user_id => user
    // keeps bill-vector
    // keeps scoring tree for saufbubbies / all users
    // keeps saufbubbies
    // keeps paginated user pages
    // keeps categorized item pages
    // keeps per user item scoring tree
    // keeps per user item simplified bill (hashmap<name,hasmap<price,number>>)
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

    fn get_purchase_timestamp(&self, purchase_id: u64) -> Option<i64> {
        return self.get_purchase(purchase_id).map(|p| *(p.get_timestamp()));
    }

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
        assert!(millis_start_inclusive <= millis_end_exclusive);

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
    fn bills_filtered(&self, user_id: Option<u32>, millis_start_inclusive: i64, millis_end_exclusive: i64) -> Vec<Bill> {
        let v : Vec<Bill> = self.bills.iter()
            .filter(|b: &&Bill| {
                matches_usergroup(&user_id, &b.users) && !(b.timestamp_to < millis_start_inclusive || b.timestamp_from > millis_end_exclusive)
            })
            .map(|p: &Bill| p.clone())
            .collect();
        return v;
    }

    fn get_mut_purchase(&mut self, id: &u64) -> Option<&mut Purchase> {
        let idx = self.purchases.binary_search_by(|p| p.get_unique_id().cmp(id));

        return idx.map(move |id| self.purchases.get_mut(id).unwrap()).ok();
    }

    fn get_mut_bill(&mut self, timestamp_from: i64, timestamp_to: i64) -> Option<&mut Bill> {
        return self.bills.iter_mut().find(|b| b.timestamp_from == timestamp_from && b.timestamp_to == timestamp_to);
    }

    fn get_bill_index(&mut self, timestamp_from: i64, timestamp_to: i64) -> Option<usize> {
        return self.bills.iter().position(|b| b.timestamp_from == timestamp_from && b.timestamp_to == timestamp_to);
    }

    fn get_specials_to_bill(&self, timestamp_from: i64, timestamp_to: i64) -> Vec<u64> {
        let (from, to) = find_purchase_indices(&self.purchases, timestamp_from, timestamp_to);
        let bill_opt = self.bills.iter().find(|b|b.timestamp_to == timestamp_to && b.timestamp_from == timestamp_from);
        if bill_opt.is_none() {
            return Vec::new();
        } else {
            let bill = bill_opt.unwrap();
            return self.purchases[from..to].iter().filter(|p| {
                p.has_user_id() && matches_usergroup(&Some(*p.get_user_id()), &bill.users) && (match **p {
                    Purchase::SpecialPurchase {
                        ..
                    } => true,
                    _ => false,
                })
            }).map(|p|p.get_unique_id()).collect();
        }
    }

    fn get_users_to_bill(&self, timestamp_from: i64, timestamp_to: i64) -> Vec<u32> {
        let bill_opt = self.bills.iter().find(|b|b.timestamp_to == timestamp_to && b.timestamp_from == timestamp_from);
        if bill_opt.is_none() {
            return Vec::new();
        } else {
            let bill = bill_opt.unwrap();

            let mut xs : Vec<u32> = vec![];

            let filtered = self.users.iter().filter(|kv| !kv.1.deleted && matches_usergroup(&Some(*kv.0), &bill.users));

            for keyvalue in filtered {
                xs.push(*keyvalue.0);
            }

                return xs;
        }
    }

    fn get_un_set_users_to_bill(&self, timestamp_from: i64, timestamp_to: i64) -> Vec<u32> {
        let bill_opt = self.bills.iter().find(|b|b.timestamp_to == timestamp_to && b.timestamp_from == timestamp_from);
        if bill_opt.is_none() {
            return Vec::new();
        } else {
            let bill = bill_opt.unwrap();


            let mut touched_users_set: HashSet<u32> = HashSet::new();
            let mut users_undefined_indices: Vec<u32> = vec![];


            for purchase in self.global_log_filtered(timestamp_from, timestamp_to) {
                let uid: u32 = *purchase.get_user_id();
                if matches_usergroup(&Some(uid), &bill.users) {
                    if !touched_users_set.contains(&uid) {
                        //user matches criteria & isn't in list => add user to list
                        touched_users_set.insert(uid);
                        let usr = self.users.get(&uid).unwrap();

                        if !usr.is_billed {
                            //if user isn't billed per field, add to externally excluded list
                        } else if bill.users_that_will_not_be_billed.contains(&uid) {
                            //else if user is in internal exclusion list of bill, add to internally excluded list
                        } else if usr.external_user_id.is_none() {
                            // else add user to other list
                            users_undefined_indices.push(uid);
                        } else if usr.external_user_id.is_some() {
                        }
                    }
                }
            }

            return users_undefined_indices;
        }
    }
    fn get_unpriced_specials_to_bill(&self, timestamp_from: i64, timestamp_to: i64) -> Vec<u64>
        {

            let mut xs : Vec<u64> = Vec::new();
            {
                use datastore::PurchaseFunctions;

                self.get_specials_to_bill(timestamp_from, timestamp_to).iter().map(|id| self.get_purchase(*id).unwrap()).filter(|p|p.get_special_set_price().is_none()).for_each(|p| xs.push(p.get_unique_id()));
            }
            return xs;
        }
    fn get_bill(&self, timestamp_from: i64, timestamp_to: i64) -> Option<&Bill> {
        return self.bills.iter().find(|b| b.timestamp_from == timestamp_from && b.timestamp_to == timestamp_to);
    }


    fn get_purchase_indices_to_bill(&self, bill: &Bill) -> Vec<usize> {
        //from all purchases, get indices inside bill date
        //filter by donor / consumer being inside bill
        let (from, to) = find_purchase_indices(&self.purchases, bill.timestamp_from, bill.timestamp_to);

        let mut v: Vec<usize> = vec![];
        for i in from..to {
            if matches_usergroup(&Some(*self.purchases[i].get_user_id()), &bill.users)  {
                v.push(i);
            }
        }
        return v;
    }

    fn remove_purchases_indices(&mut self, mut indices: Vec<usize>) {
        indices.sort();
        for idx in indices.iter().rev() {
            self.purchases.remove(*idx);
        }
    }
    fn get_budget_freeby_id_useable_for(&self, recipient_id: u32) -> Option<usize> {
        for (idx, freeby) in self.open_freebies.get(&recipient_id).unwrap_or(&Vec::new()).iter().enumerate() {
            match *freeby {
                Freeby::Transfer { .. } => {
                  return Some(idx);
                },
                _ => (),
            }
        }
        return None;
    }

    fn get_count_freeby_id_useable_for(&self, recipient_id: u32, item : u32) -> Option<usize> {
        let cat : Option<String> = self.items.get(&item).unwrap().clone().category;
        for (idx, freeby) in self.open_freebies.get(&recipient_id).unwrap_or(&Vec::new()).iter().enumerate() {
            match *freeby {
                Freeby::Classic { ref allowed_categories,
                    ref allowed_drinks, .. } => {
                    let cat = cat.clone();
                    if allowed_drinks.contains(&item) {
                        return Some(idx);
                    } else {
                        match cat {
                            Some(c) => {
                                if allowed_categories.contains(&c) {
                                    return Some(idx);
                                } else {
                                    ()
                                }
                            },
                            None => (),
                        }
                    }
                },
                _ => (),
            }
        }
        return None;
    }
    fn get_ffa_freeby(&self, id: u64) -> Option<&Freeby> {
        let found_open = self.open_ffa.binary_search_by(|f|f.get_id().cmp(&id));
        if found_open.is_ok() {
            let found = found_open.unwrap();
            return self.open_ffa.get(found);
        } else {
            let found_closed = self.used_up_freebies.binary_search_by(|f|f.get_id().cmp(&id));
            if found_closed.is_ok() {
                let found = found_closed.unwrap();
                return self.used_up_freebies.get(found);
            } else {
                return None;
            }
        }
    }

    fn get_personal_freeby(&self, recipient_id: u32, freeby_id: u64) -> Option<&Freeby> {
        let found_closed = self.used_up_freebies.binary_search_by(|f|f.get_id().cmp(&freeby_id));
        if found_closed.is_ok() {
            return self.used_up_freebies.get(found_closed.unwrap());
        } else {
            if self.open_freebies.contains_key(&recipient_id) {
                let found_open = self.open_freebies.get(&recipient_id).unwrap().binary_search_by(|f| f.get_id().cmp(&freeby_id));
                if found_open.is_ok() {
                    return self.open_freebies.get(&recipient_id).unwrap().get(found_open.unwrap());
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
    }
}


pub fn matches_usergroup(user_id: &Option<u32>, usergroup: &UserGroup) -> bool {
    if user_id.is_some() {
        let checked_user_id = user_id.clone().unwrap();
        return match *usergroup {
            UserGroup::SingleUser {
                ref user_id
            } => *user_id == checked_user_id,
            UserGroup::AllUsers => true,
            UserGroup::MultipleUsers {
                ref user_ids
            } => user_ids.iter().any(|id| *id == checked_user_id),
        }
    } else {
        return true;
    }
}


fn matches_userset(user_id: &Option<u32>, usergroup: &HashSet<u32>) -> bool {
    return user_id.map(|id| usergroup.contains(&id)).unwrap_or(true);
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
    fn get_purchase_mut(&mut self, id: u64) -> Option<&mut Purchase>;
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
        match self.purchases.binary_search_by(|p|p.get_unique_id().cmp(&id)) {
            Ok(idx) => self.purchases.get(idx).map(|p|p.clone()),
            _ => None,
        }
    }
    fn get_purchase_mut(&mut self, id: u64) -> Option<&mut Purchase> {
        return self.purchases.iter_mut().find(|p|{
            p.has_unique_id(id)
        });
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
            highlighted_users: HashSet::new(),
            top_drinks_per_user: HashMap::new(),
            drink_scores_per_user: HashMap::new(),
            balance_cost_per_user: HashMap::new(),
            balance_count_per_user: HashMap::new(),
            used_up_freebies: Vec::new(),
            open_freebies: HashMap::new(),
            open_ffa: Vec::new(),
            user_id_counter: 0,
            freeby_id_counter: 0,
            item_id_counter: 0,
            categories: HashSet::new(),
            version: 0,
        };
    }
}



#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, TypeScriptify)]
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


#[derive(Default, Debug, Serialize, Deserialize, PartialEq, TypeScriptify)]
pub struct User {
    pub username: String,
    pub external_user_id: Option<String>,
    pub user_id: u32,
    pub is_billed: bool,
    pub highlight_in_ui: bool,
    pub deleted: bool,
}

impl Clone for User {
    fn clone(&self) -> Self {
        return User {
            username: self.username.to_string(),
            external_user_id: self.external_user_id.clone(),
            user_id: self.user_id,
            is_billed: self.is_billed,
            highlight_in_ui: self.highlight_in_ui,
            deleted: self.deleted,
        };
    }
}



#[derive(Default, Debug, Serialize, Deserialize, Clone, TypeScriptify)]
pub struct Item {
    pub name: String,
    pub item_id: u32,
    pub category: Option<String>,
    pub cost_cents: u32,
    pub deleted: bool,
}



#[derive(Debug, Serialize, Deserialize, Clone, TypeScriptify)]
pub enum BillState {
    Created,
    Finalized,
    ExportedAtLeastOnce,
}

impl BillState {
    pub fn is_created(&self) -> bool {
        match *self {
            BillState::Created => return true,
            _ => return false,
        }
    }
    pub fn is_finalized(&self) -> bool {
        return !self.is_created()
    }
}

impl Default for BillState {
    fn default() -> Self {
        return BillState::Created;
    }
}



//for every user in bill the username and id at finalization, plus info if invoice or not, and hashmaps for count of all item ids, also list of specials with pricing for each, and also hashmaps for budgets, hashmap for ffa, and hashmap for count giveouts paid for by others
//usage is grouped by day
#[derive(Default, Debug, Serialize, Deserialize, Clone, TypeScriptify)]
pub struct BillUserInstance {
    pub user_id: u32,
    pub per_day: HashMap<usize, BillUserDayInstance>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, TypeScriptify)]
pub struct BillUserDayInstance {
    //pub begin_inclusive: i64,
    //pub end_exclusive: i64,

    pub personally_consumed: HashMap<u32, u32>,
    pub specials_consumed: Vec<PricedSpecial>,

    pub ffa_giveouts : HashMap<u32, u32>,
    pub giveouts_to_user_id: HashMap<u32, PaidFor>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, TypeScriptify)]
pub struct ExportableBillData {
    pub all_users : HashMap<u32, User>,
    pub all_items : HashMap<u32, Item>,
    pub user_consumption: HashMap<u32, BillUserInstance>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, TypeScriptify)]
pub struct PricedSpecial {
    pub purchase_id: u64,
    pub price: u32,
    pub name: String,
}

//one instance per user and per person that was given out to (per budget or count giveout)
#[derive(Default, Debug, Serialize, Deserialize, Clone, TypeScriptify)]
pub struct PaidFor {
    pub recipient_id: u32,
    pub count_giveouts_used: HashMap<u32,u32>,
    pub budget_given: u64,
    pub budget_gotten: u64,
}


#[derive(Default, Debug, Serialize, Deserialize, Clone, TypeScriptify)]
pub struct Bill {
    //set at creation
    pub timestamp_from: i64,
    pub timestamp_to: i64,
    pub comment: String,
    pub users: UserGroup,
    pub bill_state: BillState,

    //set between creation and finalization
    pub users_that_will_not_be_billed: HashSet<u32>,

    //set at finalization
    pub finalized_data : ExportableBillData,

}

impl Bill {
    pub fn get_day_index(&self, time: i64) -> usize {
        //TODO: deal with real date mechanics here to get correct day at 00:00.000 for every day
        let day_length: i64 = 1000i64 * 3600i64 * 24i64;
        let div: i64 = self.timestamp_from / day_length;
        let first_day_begin: i64 = div * day_length;
        let day = (time - first_day_begin) / day_length;
        return day as usize;
    }
}


pub trait PurchaseFunctions {
    fn get_unique_id(&self) -> u64;
    fn has_unique_id(&self, other: u64) -> bool;
    fn get_user_id(&self) -> &u32;
    fn has_user_id(&self) -> bool;
    fn get_item_id(&self) -> &u32;
    fn has_item_id(&self) -> bool;
    fn get_timestamp(&self) -> &i64;
    fn get_special_set_price(&self) -> Option<u32>;
    fn is_special(&self) -> bool;
}





#[derive(Debug, Serialize, Deserialize, Clone, TypeScriptify)]
pub enum Freeby {
    FFA {
        id: u64,
        allowed_categories : Vec<String>,
        allowed_drinks : Vec<u32>,
        allowed_number_total : u16,
        allowed_number_used : u16,
        text_message : String,
        created_timestamp : i64,
        donor : u32,
    },
    Transfer {
        id: u64,
        cents_worth_total : u64,
        cents_worth_used : u64,
        text_message : String,
        created_timestamp : i64,
        donor : u32,
        recipient : u32,
    },
    Classic {
        id: u64,
        allowed_categories : Vec<String>,
        allowed_drinks : Vec<u32>,
        allowed_number_total : u16,
        allowed_number_used : u16,
        text_message : String,
        created_timestamp : i64,
        donor : u32,
        recipient : u32,
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
                ref id,
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
                ref id,
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
                ref id,
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
    fn get_id(&self) -> u64 {
        return match *self {
            Freeby::Classic {
                ref id,
                ref allowed_categories,
                ref allowed_drinks,
                ref allowed_number_total,
                ref allowed_number_used,
                ref text_message,
                ref created_timestamp,
                ref donor,
                ref recipient
            } => {
                *id
            },
            Freeby::Transfer{
                ref id,
                ref cents_worth_total,
                ref cents_worth_used,
                ref text_message,
                ref created_timestamp,
                ref donor,
                ref recipient
            } => {
                *id
            },
            Freeby::FFA {
                ref id,
                ref allowed_categories,
                ref allowed_drinks,
                ref allowed_number_total,
                ref allowed_number_used,
                ref text_message,
                ref created_timestamp,
                ref donor
            } => {
                *id
            },
        };
    }

    fn get_donor(&self) -> u32 {
        return match *self {
            Freeby::Classic {
                ref id,
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
                ref id,
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
                ref id,
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
                ref id,
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
                ref id,
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
                ref id,
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
                ref id,
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
                ref id,
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
                ref id,
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
                ref id,
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
                ref id,
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
                ref id,
                ref allowed_categories,
                ref allowed_drinks,
                ref allowed_number_total,
                ref allowed_number_used,
                ref text_message,
                ref created_timestamp,
                ref donor
            } => {
                println!("total = {} & used = {}", allowed_number_total, allowed_number_used);
                (*allowed_number_total) -  (*allowed_number_used)
            },
        };
    }
    fn decrement(&mut self) -> () {
        return match *self {
            Freeby::Classic {
                ref id,
                ref allowed_categories,
                ref allowed_drinks,
                ref allowed_number_total,
                ref mut allowed_number_used,
                ref text_message,
                ref created_timestamp,
                ref donor,
                ref recipient
            } => {
                let old: u16 = *allowed_number_used;
                *allowed_number_used = old + 1;
            },
            Freeby::Transfer{
                ref id,
                ref cents_worth_total,
                ref cents_worth_used,
                ref text_message,
                ref created_timestamp,
                ref donor,
                ref recipient
            } => {
                unimplemented!();
            },
            Freeby::FFA {
                ref id,
                ref allowed_categories,
                ref allowed_drinks,
                ref allowed_number_total,
                ref mut allowed_number_used,
                ref text_message,
                ref created_timestamp,
                ref donor
            } => {
                let old: u16 = *allowed_number_used;
                *allowed_number_used = old + 1;
            },
        };
    }
    fn get_budget_cents_left(&self) -> u64 {
        return match *self {
            Freeby::Transfer{
                ref id,
                ref cents_worth_total,
                ref cents_worth_used,
                ref text_message,
                ref created_timestamp,
                ref donor,
                ref recipient
            } => {
                (*cents_worth_total - *cents_worth_used)
            },
            _ => panic!("Cannot get cents left for non-budget-freeby"),
        }
    }

    fn remove_budget_by(&mut self, value: u64) {
        match *self {
            Freeby::Transfer{
                ref id,
                ref cents_worth_total,
                ref mut cents_worth_used,
                ref text_message,
                ref created_timestamp,
                ref donor,
                ref recipient
            } => {
                *cents_worth_used += value;
            },
            _ => {
                panic!("Cannot set cents left for non-budget-freeby");
            },
        }
    }
}


pub trait FreebyAble {
    fn message(&self) -> &str;
    fn get_id(&self) -> u64;
    fn get_donor(&self) -> u32;
    fn allowed_categories(&self) -> &[String];
    fn allowed_items(&self) -> &[u32];
    fn is_open(&self) -> bool {
        return FreebyAble::left(self) != 0;
    }
    fn decrement(&mut self) -> ();
    fn get_budget_cents_left(&self) -> u64;
    fn remove_budget_by(&mut self, value: u64);
    fn left(&self) -> u16;
    fn allows(&self, item_to_allow : &Item) -> bool {
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



#[derive(Debug, Serialize, Deserialize, Clone, TypeScriptify)]
pub enum Purchase {
    FFAPurchase {
        unique_id: u64,
        timestamp_epoch_millis: i64,
        item_id: u32,
        freeby_id: u64,
        donor: u32,
    },
    //UndoPurchase { unique_id: u64 }, //deletes instance directly from purchases
    SimplePurchase {
        unique_id: u64,
        timestamp_epoch_millis: i64,
        item_id: u32, //buys one instance of this item
        consumer_id: u32,
    },
    SpecialPurchase {
        unique_id: u64,
        timestamp_epoch_millis: i64,
        special_name: String,
        specialcost: Option<u32>, //set to None, set to correct value during bill finalization
        consumer_id: u32,
    },
}


impl PurchaseFunctions for Purchase {
    fn get_unique_id(&self) -> u64 {
        match self {
            &Purchase::SpecialPurchase{
                ref unique_id,
                ref timestamp_epoch_millis,
                ref special_name,
                ref specialcost,
                ref consumer_id,
            } => {
                return *unique_id;
            },
            &Purchase::SimplePurchase  {
                ref unique_id,
                ref timestamp_epoch_millis,
                ref item_id,
                ref consumer_id,
            } => {
                return *unique_id;
            },
            &Purchase::FFAPurchase {
                ref unique_id,
                ref timestamp_epoch_millis,
                ref item_id,
                ref freeby_id,
                ref donor,
            } => {
                return *unique_id;
            },
        }
    }

    fn get_special_set_price(&self) -> Option<u32> {
        match self {
            &Purchase::SpecialPurchase {
                ref unique_id,
                ref timestamp_epoch_millis,
                ref special_name,
                ref specialcost,
                ref consumer_id,
            } => return *specialcost,
            _ => panic!("get_special_set_price called on non-special purchase"),
        }
    }

    fn has_unique_id(&self, other: u64) -> bool {
        match self {
            &Purchase::SpecialPurchase{
                ref unique_id,
                ref timestamp_epoch_millis,
                ref special_name,
                ref specialcost,
                ref consumer_id,
            } => {
                return *unique_id == other;
            },
            &Purchase::SimplePurchase {
                ref unique_id,
                ref timestamp_epoch_millis,
                ref item_id,
                ref consumer_id,
            } => {
                return *unique_id == other;
            },
            &Purchase::FFAPurchase {
                ref unique_id,
                ref timestamp_epoch_millis,
                ref item_id,
                ref freeby_id,
                ref donor,
            } => {
                return *unique_id == other;
            }
        }
    }

    fn has_user_id(&self) -> bool {
        match self {
            &Purchase::SpecialPurchase{
                ref unique_id,
                ref timestamp_epoch_millis,
                ref special_name,
                ref specialcost,
                ref consumer_id,
            } => {
                return true;
            },
            &Purchase::SimplePurchase {
                ref unique_id,
                ref timestamp_epoch_millis,
                ref item_id,
                ref consumer_id,
            } => {
                return true;
            },
            &Purchase::FFAPurchase {
                ref unique_id,
                ref timestamp_epoch_millis,
                ref item_id,
                ref freeby_id,
                ref donor,
            } => {
                return true;
            }
        }
    }
    fn get_user_id(&self) -> &u32 {
        match self {
            &Purchase::SpecialPurchase{
                ref unique_id,
                ref timestamp_epoch_millis,
                ref special_name,
                ref specialcost,
                ref consumer_id,
            } => {
                return consumer_id;
            },
            &Purchase::SimplePurchase {
                ref unique_id,
                ref timestamp_epoch_millis,
                ref item_id,
                ref consumer_id,
            } => {
                return consumer_id;
            },
            &Purchase::FFAPurchase {
                ref unique_id,
                ref timestamp_epoch_millis,
                ref item_id,
                ref freeby_id,
                ref donor,
            } => {
                return donor;
            }
        }
    }

    fn get_item_id(&self) -> &u32 {
        match self {
            &Purchase::SpecialPurchase{
                ref unique_id,
                ref timestamp_epoch_millis,
                ref special_name,
                ref specialcost,
                ref consumer_id,
            } => {
                panic!("Cannot get item_id of a special purchase");
            },
            &Purchase::SimplePurchase {
                ref unique_id,
                ref timestamp_epoch_millis,
                ref item_id,
                ref consumer_id,
            } => {
                return item_id;
            },
            &Purchase::FFAPurchase {
                ref unique_id,
                ref timestamp_epoch_millis,
                ref item_id,
                ref freeby_id,
                ref donor,
            } => {
                return item_id;
            }
        }
    }

    fn get_timestamp(&self) -> &i64 {
        match self {
            &Purchase::SpecialPurchase{
                ref unique_id,
                ref timestamp_epoch_millis,
                ref special_name,
                ref specialcost,
                ref consumer_id,
            } => {
                return timestamp_epoch_millis;
            },
            &Purchase::SimplePurchase {
                ref unique_id,
                ref timestamp_epoch_millis,
                ref item_id,
                ref consumer_id,
            } => {
                return timestamp_epoch_millis;
            },
            &Purchase::FFAPurchase {
                ref unique_id,
                ref timestamp_epoch_millis,
                ref item_id,
                ref freeby_id,
                ref donor,
            } => {
                return timestamp_epoch_millis;
            }
        }
    }
    fn has_item_id(&self) -> bool {
        match self {
            a @ &Purchase::SpecialPurchase {
                ..
            } => false,
            _ => true,
        }
    }

    fn is_special(&self) -> bool {
        match self {
            a @ &Purchase::SpecialPurchase {
                ..
            } => true,
            _ => false,
        }
    }
}
