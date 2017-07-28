// An attribute to hide warnings for unused code.
#![allow(dead_code)]


trait AVLTree {
    fn empty() -> Self;
    fn insert(&mut self, id: u32) -> ();
    fn increment_by_one(&mut self, id: u32);
    fn remove(&mut self, id: u32) -> ();
}