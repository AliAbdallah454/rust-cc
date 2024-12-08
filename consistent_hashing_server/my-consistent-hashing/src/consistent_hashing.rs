use std::{collections::{BTreeMap, HashSet}, hash::{DefaultHasher, Hash, Hasher}};

use crate::transaction::Transaction;

pub struct ConsistentHashing {
    pub ring: BTreeMap<u64, String>,
    pub nodes: HashSet<String>,
    pub virtual_nodes_count: u32,
}

#[derive(Debug)]
pub enum ConsistentHashingError {
    NodeAlreadyExists(String),
    NodeDoesNotExist(String),
    RingIsEmpty(String),
    ZeroVirtualNodes(String),
    UnchangedVirtualNodeCount(String)
}

impl ConsistentHashing {
    
    pub fn new(virtual_nodes_count: u32) -> Self {
        return ConsistentHashing {
            ring: BTreeMap::new(),
            nodes: HashSet::new(),
            virtual_nodes_count,
        };
    }

    pub fn new_with_nodes(virtual_nodes_count: u32, nodes: Vec<String>) -> Self {
        let mut consistent_hashing = ConsistentHashing::new(virtual_nodes_count);
        for node in nodes {
            match consistent_hashing.add_node(&node) {
                Ok(_) => (),
                Err(_) => panic!("Node already exists")
            };
        }
        return consistent_hashing;
    }

    pub fn get_virtual_node_form(&self, node: &str, i: u32) -> String {
        return format!("{}-{}", node, i);
    }

    pub fn get_current_state(&self) -> Vec<(u64, String)> {
        let mut x: Vec<(u64, String)> = self.ring.iter().map(|(k, v)| (*k, v.clone())).collect();
        x.sort_by(|a, b| a.0.cmp(&b.0));
        return x;
    }

    pub fn hash<U: Hash>(&self, item: &U) -> u64 {
        // let begin = Instant::now();
        let mut hasher = DefaultHasher::default();
        // println!("Hashing took {:?}", begin.elapsed());
        item.hash(&mut hasher);
        return hasher.finish();
    }

    pub fn get_previous_node(&self, node: &str) -> Option<(&u64, &String)> {
        
        let hashed_value = self.hash(&node.to_string());
        if let Some(prev) = self.ring.range(..hashed_value).next_back() {
            return Some(prev);
        }
        return self.ring.iter().next_back().clone();
    }

    pub fn get_previous_node_by_hash(&self, hash: u64) -> Option<(&u64, &String)> {
        if let Some(prev) = self.ring.range(..hash).next_back() {
            return Some(prev);
        }
        return self.ring.iter().next_back().clone();
    }

    pub fn get_next_node(&self, node: &str) -> Option<(&u64, &String)> {
        let hashed_value = self.hash(&node.to_string());
        if let Some(prev) = self.ring.range(hashed_value..).skip(1).next() {
            return Some(prev);
        }
        return self.ring.iter().next().clone();
    }

    pub fn get_next_node_by_hash(&self, hash: u64) -> Option<(&u64, &String)> {
        if let Some(prev) = self.ring.range(hash..).skip(1).next() {
            return Some(prev);
        }
        return self.ring.iter().next().clone();
    }

    /// hashes nodex-i ...
    pub fn add_node(&mut self, node: &str) -> Result<Vec<Transaction<String, u64>>, ConsistentHashingError> {
        if self.nodes.contains(node) {
            return Err(ConsistentHashingError::NodeAlreadyExists("This node already exist".to_string()));
        }

        if self.virtual_nodes_count == 0 {
            return Err(ConsistentHashingError::ZeroVirtualNodes("Cannot add node with zero virtual nodes".to_string()));
        }    
        self.nodes.insert(node.to_string());
        let mut transactions = Vec::with_capacity(self.virtual_nodes_count as usize);
        for i in 0..self.virtual_nodes_count {
            
            // let hash = self.hash(&(node.to_string(), i));
            let v_node = self.get_virtual_node_form(node, i);
            let hash = self.hash(&v_node);
            self.ring.insert(hash, node.to_string());

            let next = self.get_next_node_by_hash(hash).unwrap();
            if next.1 != node {
                let prev = self.get_previous_node_by_hash(hash).unwrap();
                let new_transaction = Transaction::new(
                    next.1.to_string(),
                    node.to_string(),
                    *prev.0,
                    hash
                );
                transactions.push(new_transaction);
            }
        }
        return Ok(transactions);
    }

    pub fn remove_node(&mut self, node: &str) -> Result<Vec<Transaction<String, u64>>, ConsistentHashingError> {
        if !self.nodes.contains(node) {
            return Err(ConsistentHashingError::NodeDoesNotExist("This node doesn't exist".to_string()));
        }

        // HashSet::with ... is not tested
        let mut seen_v_node = HashSet::with_capacity(self.virtual_nodes_count as usize);
        let mut hashes = Vec::with_capacity(self.virtual_nodes_count as usize);
        let mut transactions = vec![];
        self.nodes.remove(node);

        println!("removing: {}", node);

        for i in 0..self.virtual_nodes_count {
            
            let v_node = self.get_virtual_node_form(node, i);
            let hash = self.hash(&v_node);
            hashes.push(hash);

            if !seen_v_node.insert(hash) {
                continue;
            }

            let mut prev_node = self.get_previous_node(&v_node).expect("This should never fail. If it failed, check condition for nodes.len() > 2");
            let mut next_node = self.get_next_node(&v_node).expect("This should never fail. If it failed, check condition for nodes.len() > 2");

            while prev_node.1 == node {
                let new_hash = *prev_node.0;
                seen_v_node.insert(new_hash);
                prev_node = self.get_previous_node_by_hash(new_hash).unwrap();
            }

            if next_node.1 == node {
                let new_hash = *next_node.0;
                seen_v_node.insert(new_hash);
                next_node = self.get_next_node_by_hash(new_hash).unwrap();
            }

            let new_hash = *next_node.0;
            let final_virtual_node = self.get_previous_node_by_hash(new_hash).unwrap();

            let new_transaction = Transaction::new(
                node.to_string(),
                next_node.1.to_string(),
                *prev_node.0,
                *final_virtual_node.0
            );

            transactions.push(new_transaction);

        }

        for i in 0..self.virtual_nodes_count {
            let hash = hashes[i as usize];
            self.ring.remove(&hash);
        }
        return Ok(transactions);
    }

    pub fn set_virtual_nodes_count(&mut self, count: u32) -> Result<Vec<Transaction<String, u64>>, ConsistentHashingError> {
        
        if count == 0 {
            return Err(ConsistentHashingError::ZeroVirtualNodes("Cannot set virtual nodes count to 0".to_string()));
        }

        if count == self.virtual_nodes_count {
            return Err(ConsistentHashingError::UnchangedVirtualNodeCount("New virtual nodes count is same as current".to_string()));
        }

        let mut transactions = vec![];
        let diff: i32 = count as i32 - self.virtual_nodes_count as i32;

        if diff > 0 {
            // add nodes
            for node in &self.nodes {
                for i in self.virtual_nodes_count..count {

                    let v_node = self.get_virtual_node_form(node, i);
                    println!("adding v_node: {}", v_node);
                    let hash = self.hash(&v_node);

                    self.ring.insert(hash, node.to_string());
                    let prev = self.get_previous_node_by_hash(hash).unwrap();
                    let next = self.get_next_node_by_hash(hash).unwrap();

                    if next.1 != node {
                        let transaction = Transaction::new(
                            next.1.to_string(),
                            node.to_string(),
                            *prev.0,
                            hash
                        );
                        println!("{} with hash: {}", v_node, hash);
                        println!("trans {:?}", transaction);
                        transactions.push(transaction);
                    }


                    let state = self.get_current_state();
                    for (h, n) in state {
                        println!("{}: {}", n, h);
                    }
                }
            }
        }
        else {
            // remove nodes
            for node in &self.nodes {
                for i in (count..self.virtual_nodes_count).rev() {
                    
                    let state = self.get_current_state();
                    for (h, n) in state {
                        println!("{}: {}", n, h);
                    }
                    
                    let v_node = self.get_virtual_node_form(node, i);
                    let hash = self.hash(&v_node);

                    let prev = self.get_previous_node_by_hash(hash).unwrap();
                    let next = self.get_next_node_by_hash(hash).unwrap();

                    if next.1 != node {
                        let transaction = Transaction::new(
                            node.to_string(), 
                            next.1.to_string(),
                            *prev.0,
                            hash
                        );
                        println!("{} with hash: {}", v_node, hash);
                        println!("trans {:?}", transaction);
                        transactions.push(transaction);
                    }

                    self.ring.remove(&hash);
                }
            }
        }

        self.virtual_nodes_count = count;
        return Ok(transactions);
    }

    pub fn get_node<U: Hash>(&self, key: &U) -> (Option<&String>, Option<u64>) {
        if self.ring.is_empty() {
            return (None, None);
        }
        let hash = self.hash(key);
        println!("key hash: {}", hash);
        let node = self.ring
            .range(hash..)
            .next()
            .or_else(|| self.ring.iter().next());
        return (Some(node.unwrap().1), Some(hash));
            
    }

}