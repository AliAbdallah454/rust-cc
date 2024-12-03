#[cfg(test)]
mod tests {
    use std::hash::DefaultHasher;

    use my_consistent_hashing::consistent_hashing::ConsistentHashing;

    use super::*; // Import functions from the outer module

    #[test]
    fn test_add_node_1() {

        let mut cons = ConsistentHashing::<DefaultHasher>::new(1);

        for i in 0..5 {
            let node = format!("node{}", i);
            cons.add_node(&node).unwrap();
        }

        let state = cons.get_current_state();

        assert_eq!(state.len(), 5);

    }

}