use std::collections::HashMap;
use std::collections::hash_map;
use std::hash::Hash;
use std::ops::Index;

/// Maintain a mapping between events of type T and a count of occurrences.
#[derive(Debug)]
pub struct EventCtr<T: Eq + Hash>{ m: HashMap<T,usize>}

impl<T> EventCtr<T>
    where T: Eq + Hash {
    pub fn new() -> EventCtr<T> { EventCtr{ m: HashMap::new() } }

    pub fn inc(&mut self, k: T) {
        *self.m.entry(k).or_insert(0) += 1;
    }

    delegate!( m: pub mut entry(key:T) -> hash_map::Entry<T,usize> );
    delegate!( m: pub get(k:&T) -> Option<&usize>,
                  pub values() -> hash_map::Values<T,usize>,
                  pub iter() -> hash_map::Iter<T,usize> );
}

impl<'l, T: Eq + Hash> Index<&'l T> for EventCtr<T> {
    type Output = usize;
    delegate!( m: index(idx:&T) -> &usize );
}
