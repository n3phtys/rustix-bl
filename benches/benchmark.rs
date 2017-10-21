#![feature(test)]
extern crate rand;
extern crate test;

use test::Bencher;
use rand::Rng;
use std::mem::replace;


#[bench]
fn empty(b: &mut Bencher) {
    b.iter(|| 1)
}

#[bench]
fn setup_random_hashmap(b: &mut Bencher) {
    let mut val: u32 = 0;
    let mut rng = rand::IsaacRng::new_unseeded();
    let mut map = std::collections::HashMap::new();

    b.iter(|| {
        map.insert(rng.gen::<u8>() as usize, val);
        val += 1;
    })
}
