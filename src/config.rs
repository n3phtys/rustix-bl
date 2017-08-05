use std;

#[derive(Debug)]
pub struct StaticConfig {
    pub users_per_page: u8,
    pub users_in_top_users: u8,
    pub top_drinks_per_user: u8,
    pub database_filepath: &'static str,
}

impl Default for StaticConfig {
    fn default() -> Self {
        return StaticConfig {
            users_per_page: 20,
            users_in_top_users: 20,
            top_drinks_per_user: 4,
            database_filepath: "my_database_lmdb.db",
        };
    }
}