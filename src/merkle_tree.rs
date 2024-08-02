use sha2::{Digest, Sha256};

struct MerkleTree {
    root: Vec<u8>,
    leaves: Vec<Vec<u8>>,
}

impl MerkleTree {
    fn new(data: &[&str]) -> Self {
        let leaves: Vec<Vec<u8>> = data.iter().map(|&s| Self::hash_leaf(s)).collect();

        let root = Self::find_root(&leaves);

        Self { root, leaves }
    }

    fn find_root(leaves: &[Vec<u8>]) -> Vec<u8> {
        let mut current_level = leaves.to_vec();
        while current_level.len() > 1 {
            current_level = Self::hash_level(&current_level);
        }
        current_level.into_iter().next().unwrap()
    }

    pub fn hash_leaf(leaf: &str) -> Vec<u8> {
        return Sha256::digest(leaf.as_bytes()).to_vec();
    }

    fn hash_pair(left: &[u8], right: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().to_vec()
    }

    fn hash_level(level: &[Vec<u8>]) -> Vec<Vec<u8>> {
        level
            .chunks(2)
            .map(|chunk| match chunk {
                [left, right] => Self::hash_pair(left, right),
                [single] => single.clone(),
                _ => unreachable!(),
            })
            .collect()
    }

    fn root(&self) -> &Vec<u8> {
        &self.root
    }

    fn generate_proof(&self, leaf_index: usize) -> Vec<(Vec<u8>, bool)> {
        let mut proof = Vec::new();
        let mut current_level = self.leaves.clone();
        let mut current_index = leaf_index;

        while current_level.len() > 1 {
            let sibling = current_index ^ 1;
            if sibling < current_level.len() {
                proof.push((current_level[sibling].clone(), current_index % 2 == 0));
            }
            current_index /= 2;
            current_level = Self::hash_level(&current_level);
        }
        proof
    }

    fn verify_proof(root: &[u8], leaf: &[u8], proof: &[(Vec<u8>, bool)]) -> bool {
        let mut current_hash = leaf.to_vec();

        for (sibling, is_left) in proof {
            current_hash = if *is_left {
                Self::hash_pair(&current_hash, sibling)
            } else {
                Self::hash_pair(sibling, &current_hash)
            }
        }
        
        current_hash == root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree() {
        let data = &["a", "b", "c", "d"];
        let tree = MerkleTree::new(data);
        println!("Root: {:?}", tree.root());
        let leaf_a = MerkleTree::hash_leaf("a");
        let leaf_b = MerkleTree::hash_leaf("b");
        let leaf_c = MerkleTree::hash_leaf("c");
        let leaf_d = MerkleTree::hash_leaf("d");

        let hash_ab = MerkleTree::hash_pair(&leaf_a, &leaf_b);
        let hash_cd = MerkleTree::hash_pair(&leaf_c, &leaf_d);

        let expected_root = MerkleTree::hash_pair(&hash_ab, &hash_cd);
        println!("Expected Root: {:?}", tree.root());

        assert_eq!(tree.root(), &expected_root);
    }

    #[test]
    fn test_merkle_proof() {
        let data = &["a", "b", "c", "d"];
        let tree = MerkleTree::new(data);

        // Generate and verify proof for leaf "b" (index 1)
        let leaf_b = MerkleTree::hash_leaf("b");
        let proof = tree.generate_proof(1);
        assert!(MerkleTree::verify_proof(tree.root(), &leaf_b, &proof) == true);

        // Verify that the proof fails for a different leaf
        let leaf_c = MerkleTree::hash_leaf("c");
        assert!(MerkleTree::verify_proof(tree.root(), &leaf_c, &proof) == false);

        // Tamper with the proof and verify it fails
        let mut tampered_proof = proof.clone();
        tampered_proof[0].0[0] ^= 1; // Flip a bit in the first hash
        assert!(MerkleTree::verify_proof(tree.root(), &leaf_b, &tampered_proof) == false);
    }
}
