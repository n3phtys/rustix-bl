// An attribute to hide warnings for unused code.
#![allow(dead_code)]
#![allow(unused_parens)]


use std::collections::HashMap;

trait AVLTree {
    fn empty() -> Self;
    fn insert(&mut self, id: u32) -> bool;
    fn increment_by_one(&mut self, id: u32) -> Option<u32>; // returns score if successful
    fn remove(&mut self, id: u32) -> Option<u32>; // returns score if successful
    fn extract_top(&self, n: usize) -> Vec<u32>;
}

#[derive(Default, Builder, Debug)]
struct ScoredIdTreeMock {
    ids: Vec<u32>, //just to mock it
    scores: Vec<u32>//just to mock it
}

impl ScoredIdTreeMock {
    fn index_of(&self, id: u32) -> Option<usize> {
        for i in 0..self.ids.len() {
            if self.ids[i] == id {
              return Some(i);
            }
        }
        return None;
    }

    fn score_sorted_copy(&self) -> Vec<u32> {
        let mut hashmap : HashMap<u32,u32> = HashMap::new();
        for i in 0..self.ids.len() {
            hashmap.insert(self.ids[i], self.scores[i]);
        }
        let hm = &*(&mut hashmap);
        println!("getting:{:?}", hm);
        let cmp = |a: &u32, b: &u32|a.cmp(b);
        let mut x : Vec<u32> = self.scores.to_vec();
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
        return self.score_sorted_copy()[0..n].to_vec();
    }
}

#[cfg(test)]
mod tests {


    #[test]
    fn always_works() {
        assert!(true);
    }
    #[test]
    fn basic_adding_works() {

    }
    #[test]
    fn basic_removal_works() {

    }
}