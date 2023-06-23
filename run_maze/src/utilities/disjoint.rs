
pub struct DisjointSet {
    parent_set: Vec<usize>,
    set_rank: Vec<usize>,
}

impl DisjointSet {

    pub fn new(num_sets: usize) -> Self {
        Self {parent_set: (0..num_sets).collect(),set_rank: vec![0;num_sets]}
    }

    pub fn find(&mut self, mut id: usize) -> usize {
        let mut compress_path = Vec::new();
        while self.parent_set[id] != id {
            compress_path.push(id);
            id = self.parent_set[id];
        }
        while let Some(child) = compress_path.pop() {
            self.parent_set[child] = id;
        }
        id
    }

    pub fn made_union(&mut self, a: usize, b: usize) -> bool {
        let x = self.find(a);
        let y = self.find(b);
        if x == y {
            return false;
        }
        if self.set_rank[x] > self.set_rank[y] {
            self.parent_set[y] = x;
        } else if self.set_rank[x] < self.set_rank[y] {
            self.parent_set[x] = y;
        } else {
            self.parent_set[x] = y;
            self.set_rank[y] += 1;
        }
        return true;
    }
}
