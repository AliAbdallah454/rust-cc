use std::collections::HashSet;
use std::hash::Hasher;
use std::io::Result;
use std::time::Duration;
use std::time::Instant;

use crate::consistent_hashing::ConsistentHashing;
use crate::consistent_hashing::ConsistentHashingError;
use crate::transaction::Transaction;

impl<T: Hasher + Default> ConsistentHashing<T> {

    // fn get_next_node_by_hashv2(&self, hash: u64) -> std::collections::btree_map::Range<u64, String> {
    //     return self.ring.range(hash..);
    // }

    // fn get_previous_node_by_hashv2(&self, hash: u64) -> std::collections::btree_map::Range<u64, String> {
    //     return self.ring.range(..hash);
    // }

    // pub fn add_node_3(&mut self, node: &str) -> Result<Vec<Transaction>, ConsistentHashingError> {
    //     if self.nodes.contains(node) {
    //         return Err(ConsistentHashingError::NodeAlreadyExists("This node already exist".to_string()));
    //     }
    //     let mut transactions = Vec::with_capacity(self.virtual_nodes_count as usize);
    //     for i in 0..self.virtual_nodes_count {
            
    //         let hash = self.hash(&(node.to_string(), i));
    //         self.ring.insert(hash, node.to_string());

    //         let next = self.get_next_node_by_hash(hash).unwrap();
    //         if next.1 != node {
    //             let prev = self.get_previous_node_by_hash(hash).unwrap();
    //             let new_transaction = Transaction::new(
    //                 next.1.to_string(),
    //                 node.to_string(),
    //                 *prev.0,
    //                 hash
    //             );
    //             transactions.push(new_transaction);
    //         }
    //     }
    //     return Ok(transactions);
    // }

    // pub fn remove_node_3(&mut self, node: &str) -> Result<Vec<Transaction>, ConsistentHashingError> {
    //     if !self.nodes.contains(node) {
    //         return Err(ConsistentHashingError::NodeDoesNotExist("This node doesn't exist".to_string()));
    //     }

    //     let mut seen_v_node = HashSet::with_capacity(self.virtual_nodes_count as usize);
    //     let mut hashes = Vec::with_capacity(self.virtual_nodes_count as usize);
    //     let mut transactions = vec![];
    //     self.nodes.remove(node);

    //     println!("removing: {}", node);

    //     for i in 0..self.virtual_nodes_count {
            
    //         let v_node = self.get_virtual_node_form(node, i);
    //         let hash = self.hash(&v_node);
    //         hashes.push(hash);

    //         if !seen_v_node.insert(hash) {
    //             continue;
    //         }

    //         let mut prev_node = self.get_previous_node(&v_node).expect("This should never fail. If it failed, check condition for nodes.len() > 2");
    //         let mut next_node = self.get_next_node(&v_node).expect("This should never fail. If it failed, check condition for nodes.len() > 2");

    //         while prev_node.1 == node {
    //             let new_hash = *prev_node.0;
    //             seen_v_node.insert(new_hash);
    //             prev_node = self.get_previous_node_by_hash(new_hash).unwrap();
    //         }

    //         if next_node.1 == node {
    //             let new_hash = *next_node.0;
    //             seen_v_node.insert(new_hash);
    //             next_node = self.get_next_node_by_hash(new_hash).unwrap();
    //         }

    //         let new_hash = *next_node.0;
    //         let final_virtual_node = self.get_previous_node_by_hash(new_hash).unwrap();

    //         let new_transaction = Transaction::new(
    //             node.to_string(),
    //             next_node.1.to_string(),
    //             *prev_node.0,
    //             *final_virtual_node.0
    //         );

    //         transactions.push(new_transaction);

    //     }

    //     for i in 0..self.virtual_nodes_count {
    //         let hash = hashes[i as usize];
    //         self.ring.remove(&hash);
    //     }
    //     return Ok(transactions);
    // }

    // pub fn add_node_faster(&mut self, node: &str) -> Result<Vec<Transaction>, ConsistentHashingError> {
    //     if self.nodes.contains(node) {
    //         return Err(ConsistentHashingError::NodeAlreadyExists("This node already exist".to_string()));
    //     }

    //     if self.virtual_nodes_count == 0 {
    //         return Err(ConsistentHashingError::ZeroVirtualNodes("Cannot add node with zero virtual nodes".to_string()));
    //     }    

    //     let mut hashes = Vec::with_capacity(self.virtual_nodes_count as usize);
    //     let mut transactions = vec![];
    //     self.nodes.insert(node.to_string());

    //     for i in 0..self.virtual_nodes_count {
    //         let v_node = self.get_virtual_node_form(node, i);
    //         let hash = self.hash(&v_node);
    //         self.ring.insert(hash, node.to_string());
    //         hashes.push(hash);
    //     }

    //     if self.nodes.len() < 2 {
    //         return Ok(transactions);
    //     }

    //     let mut seen_v_node = HashSet::with_capacity(self.virtual_nodes_count as usize);

    //     for i in 0..self.virtual_nodes_count {

    //         let hash = hashes[i as usize];

    //         if !seen_v_node.insert(hash) {
    //             continue;
    //         }

    //         // Efficient but more complex implementation
    //         let mut backward_iter = self.get_previous_node_by_hashv2(hash);
    //         let mut forward_iter = self.get_next_node_by_hashv2(hash);
    //         forward_iter.next();

    //         let mut prev_node = backward_iter.next_back().or(self.ring.iter().next_back()).expect("This should never fail. If it failed, check condition for nodes.len() > 2");
    //         let mut next_node = forward_iter.next().or(self.ring.iter().next()).expect("This should never fail. If it failed, check condition for nodes.len() > 2");

    //         while prev_node.1 == node {
    //             let new_hash = *prev_node.0;
    //             seen_v_node.insert(new_hash);
    //             prev_node = backward_iter.next_back().or(self.ring.iter().next_back()).unwrap();
    //         }
    //         let mut final_virtual_node = (&0, &"".to_string());
    //         if next_node.1 == node {
    //             final_virtual_node = next_node;
    //             let new_hash = *next_node.0;
    //             seen_v_node.insert(new_hash);
    //             next_node = forward_iter.next().or(self.ring.iter().next()).unwrap();
    //         }

    //         let new_transaction = Transaction::new(
    //             next_node.1.to_string(),
    //             node.to_string(),
    //             *prev_node.0, 
    //             *final_virtual_node.0
    //         );
    //         transactions.push(new_transaction);

    //     }

    //     return Ok(transactions);
    // }

    // pub fn add_node_2(&mut self, node: &str) -> Result<Vec<Transaction>, ConsistentHashingError> {

    //     let mut transactions = Vec::with_capacity(self.virtual_nodes_count as usize);
    //     for i in 0..self.virtual_nodes_count {
            
    //         let hash = self.hash(&(node.to_string(), i));
    //         self.ring.insert(hash, node.to_string());

    //         if let Some(next) = self.ring.iter()
    //         .find(|&(_, v)| v != node && *v != node.to_string()) {
    //         let new_transaction = Transaction::new(
    //             next.1.to_string(),
    //             node.to_string(),
    //             hash,
    //             *next.0
    //         );
    //         transactions.push(new_transaction);
    //         }
    //     }
    //     return Ok(transactions);
    // }



}