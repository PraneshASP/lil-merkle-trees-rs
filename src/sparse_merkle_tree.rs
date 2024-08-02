use sha2::{Digest, Sha256};

const TREE_DEPTH: usize = 128; 

struct SparseMerkleTree {
    root: Vec<u8>,
    default_nodes: Vec<Vec<u8>>,
}

impl SparseMerkleTree {
    fn new() -> Self {
        let mut default_nodes = vec![vec![0; 32]; TREE_DEPTH + 1];
        for i in (0..TREE_DEPTH).rev() {
            default_nodes[i] = Self::hash_pair(&default_nodes[i + 1], &default_nodes[i + 1]);
        }
        Self {
            root: default_nodes[0].clone(),
            default_nodes,
        }
    }

    /**
    ===============================
    Path derivation example
    ===============================
      
     Key: [0b1101] (13 in decimal)                                                    
     Tree depth: 4 bits                                                               
                                                                                      
     Initialize:                                                                      
       path = 0b0000                                                                  
       key  = 0b1101                                                                  
                                                                                      
     i = 3 (most significant bit):                                                    
       Extract: key & (1 << 3) = 0b1101 & 0b1000 = 0b1000  // Isolate leftmost bit    
       Shift:   0b1000 << 3    = 0b1000                    // Position the bit        
       Update:  path |= 0b1000                             // Set the bit in path     
       Result:  path = 0b1000                                                         
                                                                                      
     i = 2:                                                                           
       Extract: key & (1 << 2) = 0b1101 & 0b0100 = 0b0100  // Isolate second bit       
       Shift:   0b0100 << 2    = 0b0100                        
       Update:  path |= 0b0100                             
       Result:  path = 0b1100                                                      
                                                                                  
     i = 1:                                                                        
       Extract: key & (1 << 1) = 0b1101 & 0b0010 = 0b0000  // Isolate third bit    
       Shift:   0b0000 << 1    = 0b0000                        
       Update:  path |= 0b0000                             // No change to path   
       Result:  path = 0b1100 (unchanged)                                          
                                                                                  
     i = 0 (least significant bit):                                                
       Extract: key & (1 << 0) = 0b1101 & 0b0001 = 0b0001  // Isolate rightmost bit
       Shift:   0b0001 << 0    = 0b0001                        
       Update:  path |= 0b0001                             
       Result:  path = 0b1101                                                      
                                                                                  
     Final result: path = 0b1101                                                   
                                                                                  
     This path (1101) represents the following tree traversal:                     
       1 - Go right at the first level (from the root)                             
       1 - Go right at the second level                                            
       0 - Go left at the third level                                              
       1 - Go right at the fourth level (to the leaf)                              

    */

    fn insert(&mut self, key: &[u8; 16], value: &[u8]) {
        let mut current_node = Self::hash_leaf(value);
        let mut path = 0u128;

        for i in (0..TREE_DEPTH).rev() {
            path |= (key[i / 8] as u128 & (1 << (i % 8))) << i;
            let sibling = if path & (1 << i) == 0 {
                &self.default_nodes[i + 1]
            } else {
                &self.default_nodes[i + 1]
            };
            current_node = if path & (1 << i) == 0 {
                Self::hash_pair(&current_node, sibling)
            } else {
                Self::hash_pair(sibling, &current_node)
            };
        }
        self.root = current_node;
    }

    

    fn generate_proof(&self, key: &[u8; 16]) -> Vec<Vec<u8>> {
        let mut proof = Vec::new();
        let mut path = 0u128;

        for i in (0..TREE_DEPTH).rev() {
            path |= (key[i / 8] as u128 & (1 << (i % 8))) << i; // see above example for details
            proof.push(self.default_nodes[i + 1].clone());
        }
        proof
    }

    fn verify_proof(&self, key: &[u8; 16], value: Option<&[u8]>, proof: &[Vec<u8>]) -> bool {
        let mut current_node = value.map_or_else(
            || self.default_nodes[TREE_DEPTH].clone(),
            |v| Self::hash_leaf(v),
        );
        let mut path = 0u128;

        for i in (0..TREE_DEPTH).rev() {
            path |= (key[i / 8] as u128 & (1 << (i % 8))) << i;
            current_node = if path & (1 << i) == 0 {
                Self::hash_pair(&current_node, &proof[TREE_DEPTH - 1 - i])
            } else {
                Self::hash_pair(&proof[TREE_DEPTH - 1 - i], &current_node)
            };
        }
        current_node == self.root
    }

    fn hash_leaf(leaf: &[u8]) -> Vec<u8> {
        Sha256::digest(leaf).to_vec()
    }

    fn hash_pair(left: &[u8], right: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_tree() -> SparseMerkleTree {
        let mut tree = SparseMerkleTree::new();

        let key1 = [0u8; 16];
        let value1 = b"value1";
        tree.insert(&key1, value1);

        let key2 = [1u8; 16];
        let value2 = b"value2";
        tree.insert(&key2, value2);

        let key3 = [2u8; 16];
        let value3 = b"value3";
        tree.insert(&key3, value3);

        tree
    }

    #[test]
    fn test_smt_insertion() {
        let tree = setup_tree();
        assert_ne!(tree.root, SparseMerkleTree::new().root);
    }

    #[test]
    fn test_inclusion_proof() {
        let tree = setup_tree();
        let key1 = [2u8; 16];
        let value1 = b"value3";

        let proof1 = tree.generate_proof(&key1);
        assert!(tree.verify_proof(&key1, Some(value1), &proof1));
    }

    #[test]
    fn test_non_inclusion_proof() {
        let tree = setup_tree();
        let non_existent_key = [2u8; 16];

        let proof_non_existent = tree.generate_proof(&non_existent_key);
        assert!(!tree.verify_proof(&non_existent_key, None, &proof_non_existent));
    }

    #[test]
    fn test_proof_fails_for_wrong_key() {
        let tree = setup_tree();
        let key1 = [0u8; 16];
        let key2 = [1u8; 16];
        let value1 = b"value1";

        let proof1 = tree.generate_proof(&key1);
        assert!(!tree.verify_proof(&key2, Some(value1), &proof1));
    }

    #[test]
    fn test_proof_fails_for_wrong_value() {
        let tree = setup_tree();
        let key1 = [0u8; 16];
        let wrong_value = b"wrong_value";

        let proof1 = tree.generate_proof(&key1);
        assert!(!tree.verify_proof(&key1, Some(wrong_value), &proof1));
    }
}
