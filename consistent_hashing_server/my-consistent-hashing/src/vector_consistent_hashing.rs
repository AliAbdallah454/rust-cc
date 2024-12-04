use std::hash::DefaultHasher;
use std::{collections::{BTreeMap, HashSet}, hash::{Hash, Hasher}};

use crate::transaction::{self, Transaction};

pub struct VectorConsistentHashing {
    ring: Vec<(u64, String)>,
    virtual_nodes_count: usize
}

fn _insert_sorted(vec: &mut Vec<(u64, String)>, value: (u64, String)) {
    match vec.binary_search_by_key(&value.0, |x| x.0) {
        Ok(pos) | Err(pos) => vec.insert(pos, value),
    }
}

impl VectorConsistentHashing {

    pub fn new(virtual_nodes_count: usize) -> Self {
        VectorConsistentHashing {
            ring: vec![],
            virtual_nodes_count
        }
    }
    
    pub fn hash<U: Hash>(&self, item: &U) -> u64 {
        let mut hasher = DefaultHasher::default();
        item.hash(&mut hasher);
        return hasher.finish();
    }

    pub fn add_node(&mut self, node: &str) -> Vec<Transaction> {
        let mut transactions = vec![];
        for i in 0..self.virtual_nodes_count {
            let hash = self.hash(&format!("{}-{}", node, i));
            let value = (hash, node.to_string());
            let mut p = 0 as usize;
            match self.ring.binary_search_by_key(&value.0, |x| x.0) {
                Ok(pos) | Err(pos) => {
                    self.ring.insert(pos, value);
                    p = pos;
                },
            }
            let next = if p + 1 < self.ring.len() { &self.ring[p + 1] } else { &self.ring[0] };
            let transaction = Transaction::new(next.1.to_string(), node.to_string(), hash, next.0);
            transactions.push(transaction);
        }

        return transactions;

    }

}