use std::collections::HashMap;
use std::cmp::max;
use std::hash::Hash;

use bayesian_classification::event_counter::EventCtr;
use bayesian_classification::training;

// ----------------------------------------

pub type ProbabilityMap<U> = HashMap<U,f64>;
pub type ContextMap<U> = HashMap<U,Ambiguity<U>>;

// ----------------------------------------

#[derive(Debug)]
pub struct Ambiguity<U: Hash + Eq> {
    p_raw: f64,
    pos_context: ProbabilityMap<U>,
    neg_context: ProbabilityMap<U>,
}

impl<U: Hash + Eq> Ambiguity<U> {
    fn new<T,TtoU>(ctr: &training::CtxCounter<T>,
                       event_counts: &EventCtr<T>,
                       to_u: &TtoU) -> Ambiguity<U>
        where T: Hash + Eq,
              TtoU: Fn(&T) -> U {
        Ambiguity {
            p_raw: ctr.base_probability(),
            pos_context: ctr.pos_context(to_u, event_counts),
            neg_context: ctr.neg_context(to_u, event_counts),
        }
    }

    fn localize<V: Hash + Eq,UtoV>(&self, convert: &UtoV) -> Ambiguity<V>
        where UtoV: Fn(&U) -> V {
        Ambiguity {
            p_raw: self.p_raw,
            pos_context: self.pos_context.iter()
                .map(|(k,&v)| (convert(k), v))
                .collect(),
            neg_context: self.neg_context.iter()
                .map(|(k,&v)| (convert(k), v))
                .collect(),
        }
    }

    fn log_likelihood(&self, p_unseen: f64, context: &[U]) -> (f64,f64) {
        let mut pos = self.p_raw.log2();
        let mut neg = (1.0 - self.p_raw).log2();
        for evt in context {
            pos += self.pos_context.get(evt).unwrap_or(&p_unseen).log2();
            neg += self.neg_context.get(evt).unwrap_or(&p_unseen).log2();
        }
        (pos,neg)
    }
}

// ----------------------------------------

#[derive(Debug)]
pub struct Model<U: Hash + Eq> {
    size: usize,
    p_unseen: f64,
    contexts: ContextMap<U>,
}

impl<U: Hash + Eq> Model<U> {
    pub fn new<T,TtoU>(trainer: &training::Trainer<T>, to_u: TtoU) -> Model<U>
        where T: Hash + Eq + Clone,
              TtoU: Fn(&T) -> U {
        Model {
            size: trainer.size(),
            p_unseen: trainer.p_unseen(),
            contexts: trainer.contexts.iter()
                .map(|(k,v)| (to_u(k), Ambiguity::new(v, trainer.seen(), &to_u))).collect(),
        }
    }

    pub fn localize<V: Hash + Eq,UtoV>(&self, convert: UtoV) -> Model<V>
        where UtoV: Fn(&U) -> V {
        Model {
            size: self.size,
            p_unseen: self.p_unseen,
            contexts: self.contexts.iter()
                .map(|(k,v)| (convert(k), v.localize(&convert)))
                .collect(),
        }
    }

    pub fn context<'l>(&self, idx: usize, v: &'l [U]) -> &'l [U] {
        let start = max(self.size, idx) - self.size;
        &v[start..idx]
    }

    pub fn log_likelihood(&self, instance: &U, context: &[U]) -> (f64,f64) {
        match self.contexts.get(instance) {
            Some(ctx) => ctx.log_likelihood(self.p_unseen, context),
            None => (0.0,1.0),
        }
    }

    pub fn is_instance(&self, instance: &U, context: &[U]) -> bool {
        let (p,n) = self.log_likelihood(instance, context);
        p > n
    }
}
