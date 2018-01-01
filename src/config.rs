use std;

#[derive(Debug)]
pub struct StaticConfig {
    pub users_per_page: usize,
    pub users_in_top_users: usize,
    pub top_drinks_per_user: usize,
    pub use_persistence: bool,
    pub persistence_file_path: String,
}

impl StaticConfig {
    pub fn default_persistence(filepath: &str) -> Self {
        return StaticConfig {
            users_per_page: 40,
            users_in_top_users: 40,
            top_drinks_per_user: 4,
            use_persistence: true,
            persistence_file_path: filepath.to_string(),
        };
    }
}

impl Default for StaticConfig {
    fn default() -> Self {
        return StaticConfig {
            users_per_page: 20,
            users_in_top_users: 20,
            top_drinks_per_user: 4,
            use_persistence: false,
            persistence_file_path: String::new(),
        };
    }
}
