use std;

#[derive(Debug)]
pub struct StaticConfig {
    pub users_per_page: u8,
    pub users_in_top_users: u8,
    pub database_filepath: Box<std::path::Path>,
}