struct MerkleTree<'a> {
    leafs: &'a [Leaf],
    root_hash: Hash,
    digest_hex: String,
    log2size: u32,
}

impl<'a> MerkleTree<'a> {
    fn new(leafs: &'a [Leaf], root_hash: Hash, log2size: u32) -> Self {
        MerkleTree {
            leafs,
            root_hash,
            digest_hex: root_hash.digest_hex(),
            log2size,
        }
    }

    fn join(&self, other_hash: Hash) -> Hash {
        self.root_hash.join(other_hash)
    }

    fn children(&self) -> Vec<Hash> {
        self.root_hash.children()
    }

    fn iterated_merkle(&self, level: u32) -> Hash {
        self.root_hash.iterated_merkle(level)
    }
}
