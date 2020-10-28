#![allow(dead_code)]

use std::collections::hash_map::{Entry, HashMap};
use std::hash::Hash;

#[derive(Debug)]
pub(crate) struct RepeatedElementResult<T> {
    first: T,
    last: T,
    length: usize,
    start: usize,
}

impl<T> RepeatedElementResult<T> {
    pub(crate) fn first(&self) -> &T {
        &self.first
    }

    pub(crate) fn last(&self) -> &T {
        &self.last
    }

    pub(crate) fn length(&self) -> usize {
        self.length
    }

    pub(crate) fn start(&self) -> usize {
        self.start
    }

    pub(crate) fn end(&self) -> usize {
        self.start + self.length
    }
}

pub(crate) fn repeated_element<I, T>(iter: I) -> Option<RepeatedElementResult<T>>
where
    I: Iterator<Item = T>,
    T: Hash + Eq + Clone,
{
    let iterator = iter.enumerate();
    let mut seen = HashMap::new();
    {
        let mut last: Option<T> = None;

        for (i, item) in iterator {
            match seen.entry(item) {
                Entry::Occupied(entry) => {
                    let s = *entry.get();
                    let rv = RepeatedElementResult {
                        first: (*entry.key()).clone(),
                        last: last.unwrap_or_else(|| entry.key().clone()),
                        length: i - s,
                        start: s,
                    };
                    return Some(rv);
                }
                Entry::Vacant(entry) => {
                    last = Some(entry.key().clone());
                    entry.insert(i);
                }
            }
        }
    }
    None
}
