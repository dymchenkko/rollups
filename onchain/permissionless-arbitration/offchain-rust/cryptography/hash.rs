use cartesi::keccak::keccak;
use std::collections::HashMap;

fn hex_from_bin(bin: &[u8]) -> String {
    assert_eq!(bin.len(), 32);
    let hex = bin.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    format!("0x{}", hex)
}

struct Hash {
    digest_hex: String,
}

impl Hash {
    fn from_digest(digest_hex: String) -> Self {
        assert_eq!(digest_hex.len(), 66);

        let internalized_hashes = &mut HashMap::new();
        if let Some(x) = internalized_hashes.get(&digest_hex) {
            return x.clone();
        }

        let h = Hash { digest_hex };
        internalized_hashes.insert(digest_hex.clone(), h.clone());
        h
    }

    fn from_digest_bin(digest_bin: &[u8]) -> Self {
        let digest_hex = hex_from_bin(digest_bin);
        Hash::from_digest(digest_hex)
    }

    fn from_data(data: &[u8]) -> Self {
        let digest_hex = keccak(data);
        Hash::from_digest(digest_hex)
    }

    fn join(&self, other_hash: &Hash) -> Hash {
        let digest_hex = keccak(&self.digest_hex, &other_hash.digest_hex);
        let ret = Hash::from_digest(digest_hex);
        ret.left = Some(self.digest_hex.clone());
        ret.right = Some(other_hash.digest_hex.clone());
        ret
    }

    fn children(&self) -> Option<(String, String)> {
        match (self.left.clone(), self.right.clone()) {
            (Some(left), Some(right)) => Some((left, right)),
            _ => None,
        }
    }

    fn iterated_merkle(&self, level: u32) -> Hash {
        let level = level + 1;
        let iterated = &mut Vec::new();
        let mut i = iterated.len();
        let mut highest_level = iterated[i].clone();
        while i < level {
            highest_level = highest_level.join(&highest_level);
            i += 1;
            iterated.push(highest_level.clone());
        }
        highest_level
    }

    fn is_zero(&self) -> bool {
        self == &ZERO_HASH
    }

    fn is_of_type_hash(&self, x: &Hash) -> bool {
        self == x
    }
}

impl PartialEq for Hash {
    fn eq(&self, other: &Self) -> bool {
        self.digest_hex == other.digest_hex
    }
}

impl Eq for Hash {}

impl std::fmt::Debug for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.digest_hex)
    }
}