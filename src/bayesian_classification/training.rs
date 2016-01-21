use std::cmp::max;
use std::collections::HashMap;
use std::hash::Hash;
use std::iter::FromIterator;

use bayesian_classification::event_counter::EventCtr;

/// Collection of statistics about a binary event.
#[derive(Debug)]
pub struct CtxCounter<T: Eq + Hash> {
    positive: usize,
    total: usize,
    pos_context: EventCtr<T>,
    neg_context: EventCtr<T>,
}

impl<T: Eq + Hash> CtxCounter<T> {

    pub fn new() -> CtxCounter<T> {
        CtxCounter {
            total: 0,
            positive: 0,
            pos_context: EventCtr::new(),
            neg_context: EventCtr::new(),
        }
    }

    // Record a boolean event and its context.
    pub fn seen<I: Iterator<Item=T>>(&mut self, context: I, pos: bool) {
        self.total += 1;
        if pos { self.positive += 1; }
        for event in context {
            if pos {
                self.pos_context.inc(event);
            } else {
                self.neg_context.inc(event);
            }
        }
    }

    // Calculate the raw probability of an event.
    pub fn base_probability(&self) -> f64 { (self.positive as f64) / (self.total as f64) }

    pub fn pos_context<K,ToK,R>(&self, f: ToK, event_counts: &EventCtr<T>) -> R
        where R: FromIterator<(K,f64)>,
              ToK: Fn(&T) -> K {
        CtxCounter::ctx_probability(&self.pos_context, event_counts, f)
    }

    pub fn neg_context<K,ToK,R>(&self, f: ToK, event_counts: &EventCtr<T>) -> R
        where R: FromIterator<(K,f64)>,
              ToK: Fn(&T) -> K {
        CtxCounter::ctx_probability(&self.neg_context, event_counts, f)
    }

    // Evidence: compute the probability of context words given an event.
    fn ctx_probability<K,ToK,R>(context: &EventCtr<T>, word_counts: &EventCtr<T>, f: ToK) -> R
        where R: FromIterator<(K,f64)>,
              ToK: Fn(&T) -> K {
        // Add-one (Laplace) smoothing.
        //
        // The original, maximum-liklihood estimate is the number of times a
        // word is used in this context divided by the number of times it is
        // used overall:
        //
        // v / word_counts[k]
        //
        // Smoothing adds one to the numerator and the total number of words
        // to the denominator:
        //
        // (v + 1) / (word_counts[k] + Sum(word_counts))
        let sum = word_counts.values().fold(0, |a,c| a+c);
        context.iter()
            .map(move |(k,&v)| {
                let numer = (v+1) as f64;
                let denom = (word_counts.get(k).unwrap_or(&0) + sum) as f64;
                (f(k), numer / denom)
            }).collect()
    }

}

// ----------------------------------------

/// Maintain a mapping between events and CtxCounters.
pub type ContextMap<T> = HashMap<T, CtxCounter<T>>;

#[derive(Debug)]
pub struct Trainer<T: Eq + Hash> {
    size: usize,
    seen: EventCtr<T>,
    pub contexts: ContextMap<T>,
}

impl<T: Eq + Hash + Clone> Trainer<T> {

    pub fn new(context_size: usize) -> Trainer<T> {
        Trainer {
            size: context_size,
            seen: EventCtr::new(),
            contexts: ContextMap::new(),
        }
    }

    pub fn train<'l,IsExample,Untag,IsTag>(&mut self,
                                           events: &'l[T],
                                           is_example: IsExample,
                                           untag: Untag,
                                           is_tag: IsTag)
        where IsExample: Fn(&T) -> bool,
              Untag: Fn(&T) -> T,
              IsTag: Fn(&T) -> bool {
        for (i,evt) in events.iter().enumerate() {
            self.seen.inc(untag(evt));
            if is_example(evt) {
                let start = max(i, self.size) - self.size;
                let context = &events[start..i];
                self.counter(&untag(evt))
                    .seen(context.iter().map(|w| untag(w)), is_tag(evt));
            }
        }
    }

    pub fn size(&self) -> usize { self.size }
    pub fn seen(&self) -> &EventCtr<T> { &self.seen }
    pub fn p_unseen(&self) -> f64 { 1.0 / (self.seen.values().fold(0, |a,c| a+c) as f64) }

    fn counter<'l>(&'l mut self, m: &T) -> &'l mut CtxCounter<T> {
        self.contexts.entry(m.clone()).or_insert_with(|| CtxCounter::new())
    }
}
