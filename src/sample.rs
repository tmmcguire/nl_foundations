use std::collections::HashMap;
use std::hash::Hash;
use std::iter::FromIterator;

/// A collection of events, with a running total of counts for each event and
/// total number of events.
pub struct Sample<T> {
    pub counts: HashMap<T,usize>,
    pub total: usize,
}

impl<T: Eq + Hash> Sample<T> {
    /// Creates a new Sample.
    pub fn new() -> Sample<T> {
        Sample {
            counts: HashMap::new(),
            total: 0,
        }
    }

    /// Add an event to a sample.
    pub fn add(&mut self, event: T) {
        let count = self.counts.entry(event).or_insert(0);
        *count += 1;
        self.total += 1;
    }

    /// The probability of an event in a sample.
    pub fn p(&self, event: &T) -> f64 {
        let c = *self.counts.get(event).unwrap_or(&0);
        (c as f64) / (self.total as f64)
    }
}

// ---------------------------

impl<T: Eq + Hash> Extend<T> for Sample<T> {
    fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I) {
        for k in iter { self.add(k); }
    }
}

impl<T: Eq + Hash> FromIterator<T> for Sample<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iterable: I) -> Sample<T> {
        let mut sample = Sample::new();
        sample.extend( iterable.into_iter() );
        sample
    }
}
