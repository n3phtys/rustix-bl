// An attribute to hide warnings for unused code.
#![allow(dead_code)]
#![allow(unused_parens)]


use std::collections::HashMap;
use std::cmp;

pub trait AVLTree {
    fn empty() -> Self;
    fn insert(&mut self, id: u32) -> bool;
    fn increment_by_one(&mut self, id: u32) -> Option<u32>; // returns score if successful
    fn remove(&mut self, id: u32) -> Option<u32>; // returns score if successful
    fn extract_top(&self, n: usize) -> Vec<u32>;
}

#[derive(Builder, Debug)]
pub struct ScoredIdTreeMock {
    ids: Vec<u32>, //just to mock it
    scores: Vec<u32>//just to mock it
}

impl Default for ScoredIdTreeMock {
    fn default() -> Self {
        return ScoredIdTreeMock{ ids: Vec::new(), scores: Vec::new() };
    }
}

impl ScoredIdTreeMock {
    fn index_of(&self, id: u32) -> Option<usize> {
        for i in 0..(self.ids.len() - 1) {
            if self.ids[i] == id {
              return Some(i);
            }
        }
        return None;
    }

    fn score_sorted_copy(&self) -> Vec<u32> {
        let mut hashmap : HashMap<u32,u32> = HashMap::new();
        for i in 0..(self.ids.len()-1) {
            hashmap.insert(self.ids[i], self.scores[i]);
        }
        let hm = &*(&mut hashmap);
        let cmp = |a: &u32, b: &u32|hm.get(b).unwrap_or(&0u32).cmp(hm.get(a).unwrap_or(&0u32));
        let mut x : Vec<u32> = self.ids.to_vec();
        x.sort_by(cmp);
        return x;
    }
}

impl AVLTree for ScoredIdTreeMock {
    fn empty() -> Self {
        return ScoredIdTreeMock::default();
    }

    fn insert(&mut self, id: u32) -> bool {
        if (!self.ids.contains(&id)) {
            self.ids.push(id);
            self.scores.push(0);
            return true;
        } else {
            return false;
        }
    }

    fn increment_by_one(&mut self, id: u32) -> Option<u32> {
        let o = self.index_of(id);
        return o.map(|i| { self.scores[i] = self.scores[i] + 1; self.scores[i]});
    }

    fn remove(&mut self, id: u32) -> Option<u32> {
        let o = self.index_of(id);
        match o {
            Some(i) => {
                self.ids.remove(i);
                return Some(self.scores.remove(i));
            }   ,
            None => {
                return None;
            },
        }
    }

    fn extract_top(&self, n: usize) -> Vec<u32> {
        let n : usize = cmp::min(n, self.ids.len());
        return self.score_sorted_copy()[0..(n)].to_vec();
    }
}

#[cfg(test)]
mod tests {

    use left_threaded_avl_tree::ScoredIdTreeMock;
    use left_threaded_avl_tree::AVLTree;

    #[test]
    fn always_works() {
        assert!(true);
    }
    #[test]
    fn basic_adding_works() {
        let mut tree = ScoredIdTreeMock::empty();
        assert!(tree.insert(1));
        assert!(!tree.insert(1));
        assert!(tree.insert(2));
        assert!(!tree.insert(1));
        assert!(tree.insert(3));
        assert_eq!(tree.increment_by_one(2).unwrap(), 1);
        assert_eq!(tree.increment_by_one(2).unwrap(), 2);
        assert_eq!(tree.increment_by_one(2).unwrap(), 3);
        assert_eq!(tree.increment_by_one(1).unwrap(), 1);
        assert!(tree.increment_by_one(0).is_none());
        let out = tree.extract_top(3);
        assert_eq!(tree.ids.len(), 3);
        assert_eq!(tree.scores.len(), 3);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0], 2);
        assert_eq!(out[1], 1);
        assert_eq!(out[2], 3);
    }
    #[test]
    fn basic_removal_works() {

        let mut tree = ScoredIdTreeMock::default();
        tree.insert(1);
        tree.insert(1);
        tree.insert(2);
        tree.insert(1);
        tree.insert(3);
        tree.increment_by_one(2);
        tree.increment_by_one(2);
        tree.increment_by_one(1);
        tree.increment_by_one(1);
        tree.increment_by_one(1);
        tree.increment_by_one(0);
        tree.increment_by_one(1);
        assert_eq!(tree.remove(2).unwrap(), 2);
        assert_eq!(tree.remove(2).is_none(), true);
        assert_eq!(tree.remove(1).unwrap(), 4);
        let out = tree.extract_top(3);
        assert_eq!(tree.ids.len(), 1);
        assert_eq!(tree.scores.len(), 1);
        assert_eq!(out.len(), 1);

    }
}