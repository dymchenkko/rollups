use cryptography::merkle_tree::MerkleTree;
use utils::arithmetic::{self, ulte, semi_sum};

struct Slice<'a> {
    arr: &'a [u8],
    start_idx_inc: usize,
    end_idx_ex: usize,
}

impl<'a> Slice<'a> {
    fn new(arr: &'a [u8], start_idx_inc: usize, end_idx_ex: usize) -> Self {
        let start_idx_inc = start_idx_inc.max(1);
        let end_idx_ex = end_idx_ex.min(arr.len() + 1);
        assert!(start_idx_inc > 0);
        assert!(ulte(start_idx_inc, end_idx_ex));
        assert!(end_idx_ex <= arr.len() + 1);
        Slice {
            arr,
            start_idx_inc,
            end_idx_ex,
        }
    }

    fn slice(&self, si: usize, ei: usize) -> Self {
        assert!(si > 0);
        assert!(ulte(si, ei));
        let start_idx_inc = self.start_idx_inc + si - 1;
        let end_idx_ex = self.start_idx_inc + ei - 1;
        assert!(ulte(end_idx_ex, self.end_idx_ex));
        Slice::new(self.arr, start_idx_inc, end_idx_ex)
    }

    fn len(&self) -> usize {
        self.end_idx_ex - self.start_idx_inc
    }

    fn get(&self, idx: usize) -> u8 {
        let idx = idx as isize;
        assert!(idx > 0);
        let i = (self.start_idx_inc as isize + idx - 1) as usize;
        assert!(i < self.end_idx_ex);
        self.arr[i]
    }

    fn find_cell_containing(&self, elem: isize) -> usize {
        let mut l = 1;
        let mut r = self.len();

        while arithmetic::ult(l, r) {
            let m = semi_sum(l, r);
            if arithmetic::ult(self.get(m) as isize - 1, elem - 1) {
                l = m + 1;
            } else {
                r = m;
            }
        }

        l
    }
}

struct MerkleBuilder {
    leafs: Vec<Leaf>,
}

impl MerkleBuilder {
    fn new() -> Self {
        MerkleBuilder { leafs: Vec::new() }
    }

    fn add(&mut self, hash: Hash, rep: usize) {
        let rep = rep.max(1);
        assert!(0 < rep);

        if let Some(last) = self.leafs.last() {
            assert!(last.accumulated_count != 0, "merkle builder is full");
            let accumulated_count = rep + last.accumulated_count;

            if !ulte(rep, accumulated_count) {
                assert_eq!(accumulated_count, 0);
            }

            self.leafs.push(Leaf {
                hash,
                accumulated_count,
            });
        } else {
            self.leafs.push(Leaf {
                hash,
                accumulated_count: rep,
            });
        }
    }

    fn build(&self) -> MerkleTree {
        let last = self.leafs.last().expect("no leafs in merkle builder");
        let count = last.accumulated_count;

        let log2size = if count == 0 {
            64
        } else {
            assert!(arithmetic::is_pow2(count), count);
            arithmetic::ctz(count)
        };

        let root_hash = merkle(&Slice::new(&self.leafs, 0, self.leafs.len()), log2size, 0);
        MerkleTree::new(&self.leafs, root_hash, log2size)
    }
}

struct Leaf {
    hash: Hash,
    accumulated_count: usize,
}

fn merkle(leafs: &Slice, log2size: u32, stride: usize) -> Hash {
    let first_time = stride * (1 << log2size) + 1;
    let last_time = (stride + 1) * (1 << log2size);

    let first_cell = leafs.find_cell_containing(first_time as isize);
    let last_cell = leafs.find_cell_containing(last_time as isize);

    if first_cell == last_cell {
        return leafs.get(first_cell).hash.iterated_merkle(log2size);
    }

    let slice = leafs.slice(first_cell, last_cell + 1);
    let hash_left = merkle(&slice, log2size - 1, stride << 1);
    let hash_right = merkle(&slice, log2size - 1, (stride << 1) + 1);

    hash_left.join(hash_right)
}