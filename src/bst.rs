extern crate alloc;
use super::{Error, Result};
const BST_MAX_SIZE: usize = 4096;

pub struct Bst<D>
where
    D: PartialOrd + Clone + Copy,
{
    data: [Option<Node<D>>; BST_MAX_SIZE],
    head: Option<usize>,
    length: usize,
}

impl<D> Bst<D>
where
    D: PartialOrd + Clone + Copy,
{
    pub fn insert(&mut self, data: D) -> Result<()> {
        let Some(index) = self.first_empty() else {
            return Err(Error::OutOfSpace);
        };

        self.data[index] = Some(Node::new(data));
        self.length += 1;

        let Some(mut current_idx) = self.head else {
            self.head = Some(index);
            return Ok(());
        };
        while let Some(node) = self.data[current_idx].as_mut() {
            if data < node.data {
                if let Some(left_idx) = node.left {
                    current_idx = left_idx;
                } else {
                    node.left = Some(index);
                    return Ok(());
                }
            } else if let Some(right_idx) = node.right {
                current_idx = right_idx;
            } else {
                node.right = Some(index);
                return Ok(());
            }
        }
        unreachable!();
    }

    fn first_empty(&self) -> Option<usize> {
        for (index, node) in self.data.iter().enumerate() {
            if node.is_none() {
                return Some(index);
            }
        }
        None
    }

    fn dfs(&self, idx: Option<usize>, values: &mut alloc::vec::Vec<D>) {
        if let Some(index) = idx {
            if let Some(node) = &self.data[index] {
                self.dfs(node.left, values);
                values.push(node.data);
                self.dfs(node.right, values);
            }
        }
    }

    pub fn search(&self, data: D) -> Option<D> {
        let mut current_idx = self.head;
        while let Some(index) = current_idx {
            if let Some(node) = &self.data[index] {
                if data == node.data {
                    return Some(node.data);
                } else if data < node.data {
                    current_idx = node.left;
                } else {
                    current_idx = node.right;
                }
            }
        }
        None
    }

    pub fn new() -> Bst<D> {
        Bst {
            data: [None; BST_MAX_SIZE],
            length: 0,
            head: None,
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }
}

#[derive(Debug, Clone, Copy)]
struct Node<D>
where
    D: PartialOrd + Clone + Copy,
{
    data: D,
    left: Option<usize>,
    right: Option<usize>,
}

impl<D> Node<D>
where
    D: PartialOrd + Clone + Copy,
{
    fn new(data: D) -> Self {
        Node {
            data,
            left: None,
            right: None,
        }
    }
}

#[cfg(test)]
mod tests {}

#[cfg(test)]
mod fuzz_tests {
    extern crate std;
    use super::Bst;
    use super::BST_MAX_SIZE;
    use rand::seq::SliceRandom;
    use rand::Rng;
    use std::collections::HashSet;
    use std::vec::Vec;

    #[test]
    fn fuzz_insert() {
        for _ in 0..100 {
            let mut bst = Bst::<usize>::new();
            let mut rng = rand::thread_rng();
            let min = 1;
            let max = 100_000;

            let mut random_numbers = HashSet::new();

            while random_numbers.len() < BST_MAX_SIZE {
                let num = rng.gen_range(min..=max);
                random_numbers.insert(num);
            }

            let mut random_numbers: Vec<_> = random_numbers.into_iter().collect();
            random_numbers.shuffle(&mut rng);

            assert_eq!(random_numbers.len(), BST_MAX_SIZE);
            for num in random_numbers.iter() {
                assert!(bst.insert(*num).is_ok());
            }

            random_numbers.sort();

            let mut ordered_numbers = Vec::new();
            bst.dfs(bst.head, &mut ordered_numbers);
            assert_eq!(ordered_numbers, random_numbers);
        }
    }

    #[test]
    fn fuzz_search() {
        let mut bst = Bst::<usize>::new();
        let mut rng = rand::thread_rng();
        let min = 50_000;
        let max = 100_000;

        let mut random_numbers = HashSet::new();
        while random_numbers.len() < BST_MAX_SIZE {
            let num = rng.gen_range(min..=max);
            random_numbers.insert(num);
        }

        let mut random_numbers: Vec<_> = random_numbers.into_iter().collect();
        random_numbers.shuffle(&mut rng);

        assert_eq!(random_numbers.len(), BST_MAX_SIZE);
        for num in random_numbers.iter() {
            assert!(bst.insert(*num).is_ok());
        }

        // Search for numbers that exist in the tree
        for _ in 0..1_000_000 {
            let num = random_numbers.choose(&mut rng).unwrap();
            assert!(bst.search(*num).is_some());
        }

        // Search for numbers that do not exist in the tree
        for _ in 0..1_000_000 {
            let to_search = rng.gen_bool(0.5);
            let random_number = if to_search {
                rng.gen_range(0..=min)
            } else {
                rng.gen_range(max..=max + 50_000)
            };
            assert!(bst.search(random_number).is_none());
        }
    }
}
