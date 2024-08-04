## Lil Merkle Trees 🌴

Merkle trees are one of the most widely used data structure in the blockchain ecosystem. I've seen different modifications of this data structure based on the usecase. 
This repo consists of minimal implementation of merkle tree variations, implemented in pure rust. 

- [x] [Merkle tree](./src/merkle_tree.rs) - Can be used for Airdrops
- [X] [Sparse Merkle tree](./src/sparse_merkle_tree.rs) - Non-inclusion proofs
- [X] [Merkle Mountain ranges](./src/merkle_mountain_ranges.rs) - Opentimestamp, Axiom
- [ ] Merkle practicia trie - Ethereum 
- [ ] Urkle Tree - Handshake protocol