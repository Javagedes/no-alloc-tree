extern crate alloc;
use core::ptr::null_mut;
use core::{
    mem::size_of,
    panic, slice,
    sync::atomic::{AtomicPtr, Ordering},
};

use super::{Error, Result};

pub const fn node_size<D: core::cmp::PartialOrd>() -> usize {
    size_of::<(bool, Node<D>)>()
}

pub trait BstKey {
    type Key: Ord;
    fn ordering_key(&self) -> &Self::Key;
}

impl<T> BstKey for T
where
    T: Ord,
{
    type Key = Self;
    fn ordering_key(&self) -> &T {
        self
    }
}

pub struct Storage<'a, D, const SIZE: usize>
where
    D: PartialOrd,
{
    pub data: &'a mut [(bool, Node<D>)],
    pub length: usize,
    free_indices: arrayvec::ArrayVec<u16, SIZE>,
}

impl<'a, D, const SIZE: usize> Storage<'a, D, { SIZE }>
where
    D: PartialOrd + core::fmt::Debug,
{
    /// Create a new storage container.
    fn new(slice: &'a mut [u8]) -> Storage<'a, D, SIZE> {
        Storage {
            data: unsafe {
                slice::from_raw_parts_mut::<'a, (bool, Node<D>)>(
                    slice as *mut [u8] as *mut (bool, Node<D>),
                    SIZE,
                )
            },
            length: 0,
            free_indices: arrayvec::ArrayVec::from(array_init::array_init(|i| i as u16)),
        }
    }

    /// Add a new node to the storage container, returning a mutable reference to the node.
    fn add(&mut self, data: D) -> Result<&mut Node<D>> {
        if let Some(index) = self.free_indices.pop() {
            self.data[index as usize] = (true, Node::new(data));

            let (_, node) = self.data.get_mut(index as usize).unwrap();
            self.length += 1;
            return Ok(node);
        }
        Err(Error::OutOfSpace)
    }

    /// Delete a node from the storage container.
    fn delete(&mut self, ptr: *mut Node<D>) {
        // Calculate the index of the node in the storage container based off the pointer.
        let index =
            (ptr as usize - self.data.as_ptr() as usize) / core::mem::size_of::<(bool, Node<D>)>();
        self.data[index].0 = false;
        self.length -= 1;
        self.free_indices.push(index as u16);
    }
}

pub struct Bst<'a, D, const SIZE: usize>
where
    D: PartialOrd,
{
    pub storage: Storage<'a, D, SIZE>,
    pub head: AtomicPtr<Node<D>>,
}

impl<'a, D, const SIZE: usize> Bst<'a, D, { SIZE }>
where
    D: PartialOrd + Copy + core::fmt::Debug + BstKey,
{
    pub fn new(slice: &'a mut [u8]) -> Self {
        Self {
            storage: Storage::new(slice),
            head: AtomicPtr::default(),
        }
    }

    pub fn head(&self) -> Option<&Node<D>> {
        let head_ptr = self.head.load(Ordering::SeqCst);
        if head_ptr.is_null() {
            return None;
        }
        Some(unsafe { &*head_ptr })
    }

    pub fn insert(&mut self, data: D) -> Result<()> {
        let node = self.storage.add(data)?;

        if self.head.load(Ordering::SeqCst).is_null() {
            self.head.store(node.as_mut_ptr(), Ordering::SeqCst);
            return Ok(());
        }

        let head = unsafe { &*self.head.load(Ordering::SeqCst) };
        let mut current = head;
        loop {
            if node.data < current.data {
                match current.left() {
                    Some(left) => current = left,
                    None => {
                        current.set_left(node.as_mut_ptr());
                        node.set_parent(current);
                        return Ok(());
                    }
                }
            } else if node.data > current.data {
                match current.right() {
                    Some(right) => current = right,
                    None => {
                        current.set_right(node.as_mut_ptr());
                        node.set_parent(current);
                        return Ok(());
                    }
                }
            } else {
                panic!("Duplicate data found in the tree");
            }
        }
    }

    pub fn search(&self, key: &D::Key) -> Option<D> {
        self.search_node(key).map(|node| node.data)
    }

    fn search_node(&self, key: &D::Key) -> Option<&Node<D>> {
        let mut current = self.head();
        while let Some(node) = current {
            if key < node.data.ordering_key() {
                current = node.left();
            } else if key > node.data.ordering_key() {
                current = node.right();
            } else {
                return Some(node);
            }
        }
        None
    }

    fn replace_node(head: &AtomicPtr<Node<D>>, old: *mut Node<D>, new: *mut Node<D>) {
        if let Some(parent) = unsafe { &*old }.parent() {
            if parent.left_ptr() == old {
                parent.set_left(new);
            } else if parent.right_ptr() == old {
                parent.set_right(new);
            } else {
                panic!("BST is corrupted. Parent does not point to child");
            }

            if !new.is_null() {
                unsafe { &*new }.set_parent(parent);
            }
        // If the old node has no parent, it is the head of the tree
        } else if !new.is_null() {
            head.store(new, Ordering::SeqCst);
            if !new.is_null() {
                unsafe { &*new }.set_parent(null_mut());
            }
        }
    }

    pub fn delete(&mut self, data: D) -> Result<()> {
        let Some(to_delete) = self.search_node(data.ordering_key()) else {
            return Err(Error::NotFound);
        };

        let left = to_delete.left();
        let right = to_delete.right();

        // Node has no children, unlink from parent and delete
        if left.is_none() && right.is_none() {
            Self::replace_node(&self.head, to_delete.as_mut_ptr(), null_mut());
        }
        // Node only has one child (right)
        else if left.is_none() {
            Self::replace_node(
                &self.head,
                to_delete.as_mut_ptr(),
                right.unwrap().as_mut_ptr(),
            );
        }
        // Node only has one child (left)
        else if right.is_none() {
            Self::replace_node(
                &self.head,
                to_delete.as_mut_ptr(),
                left.unwrap().as_mut_ptr(),
            );
        }
        // Node has both children
        else {
            let left = left.unwrap();
            let right = right.unwrap();
            // find the in-order successor - left most child of the right subtree
            let mut successor = right;
            while let Some(left) = successor.left() {
                successor = left;
            }

            // If the successor is not the right child, replace the successor with it's right child
            if successor.as_mut_ptr() != right.as_mut_ptr() {
                Self::replace_node(&self.head, successor.as_mut_ptr(), successor.right_ptr());
                successor.set_right(right);
                right.set_parent(successor);
            }
            Self::replace_node(&self.head, to_delete.as_mut_ptr(), successor.as_mut_ptr());
            successor.set_left(left);
            left.set_parent(successor);
        }

        self.storage.delete(to_delete.as_mut_ptr());
        Ok(())
    }

    #[allow(dead_code)]
    fn dfs(&self, node: Option<&Node<D>>, values: &mut alloc::vec::Vec<D>) {
        if let Some(node) = node {
            self.dfs(node.left(), values);
            values.push(node.data);
            self.dfs(node.right(), values);
        }
    }
}

#[derive(Debug)]
pub struct Node<D>
where
    D: PartialOrd,
{
    data: D,
    parent: AtomicPtr<Node<D>>,
    left: AtomicPtr<Node<D>>,
    right: AtomicPtr<Node<D>>,
}

impl<D> Node<D>
where
    D: PartialOrd,
{
    fn new(data: D) -> Self {
        Node {
            data,
            parent: AtomicPtr::default(),
            left: AtomicPtr::default(),
            right: AtomicPtr::default(),
        }
    }

    fn right(&self) -> Option<&Node<D>> {
        let node = self.right.load(Ordering::SeqCst);
        if node.is_null() {
            return None;
        }
        Some(unsafe { &*node })
    }

    fn right_ptr(&self) -> *mut Node<D> {
        self.right.load(Ordering::SeqCst)
    }

    fn set_right<N: Into<*mut Node<D>>>(&self, node: N) {
        self.right.store(node.into(), Ordering::SeqCst);
    }

    fn left(&self) -> Option<&Node<D>> {
        let node = self.left.load(Ordering::SeqCst);
        if node.is_null() {
            return None;
        }
        Some(unsafe { &*node })
    }

    fn left_ptr(&self) -> *mut Node<D> {
        self.left.load(Ordering::SeqCst)
    }

    fn set_left<N: Into<*mut Node<D>>>(&self, node: N) {
        self.left.store(node.into(), Ordering::SeqCst);
    }

    fn parent(&self) -> Option<&Node<D>> {
        let node = self.parent.load(Ordering::SeqCst);
        if node.is_null() {
            return None;
        }
        Some(unsafe { &*node })
    }

    #[allow(dead_code)]
    fn parent_ptr(&self) -> *mut Node<D> {
        self.parent.load(Ordering::SeqCst)
    }

    fn set_parent<N: Into<*mut Node<D>>>(&self, node: N) {
        self.parent.store(node.into(), Ordering::SeqCst);
    }

    pub fn as_mut_ptr(&self) -> *mut Node<D> {
        self as *const _ as *mut _
    }
}

impl<D> From<&Node<D>> for *mut Node<D>
where
    D: PartialOrd,
{
    fn from(node: &Node<D>) -> *mut Node<D> {
        node.as_mut_ptr()
    }
}

#[cfg(test)]
mod tests {}

#[cfg(test)]
mod fuzz_tests {
    extern crate std;
    use super::{node_size, Bst};
    use rand::seq::SliceRandom;
    use rand::Rng;
    use std::collections::HashSet;
    use std::vec::Vec;

    const BST_MAX_SIZE: usize = 4096;

    #[test]
    fn fuzz_insert() {
        for _ in 0..100 {
            let mut mem = [0; BST_MAX_SIZE * node_size::<i32>()];
            let mut bst: Bst<i32, BST_MAX_SIZE> = Bst::new(&mut mem);
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
            bst.dfs(bst.head(), &mut ordered_numbers);
            assert_eq!(ordered_numbers, random_numbers);
        }
    }

    #[test]
    fn fuzz_search() {
        let mut mem = [0; BST_MAX_SIZE * node_size::<i32>()];
        let mut bst: Bst<i32, BST_MAX_SIZE> = Bst::new(&mut mem);
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
        for _ in 0..100_000 {
            let num = random_numbers.choose(&mut rng).unwrap();
            assert!(bst.search(num).is_some());
        }

        // Search for numbers that do not exist in the tree
        for _ in 0..100_000 {
            let to_search = rng.gen_bool(0.5);
            let random_number = if to_search {
                rng.gen_range(0..=min - 1)
            } else {
                rng.gen_range(max + 1..=max + 50_000)
            };
            assert!(bst.search(&random_number).is_none());
        }
    }

    #[test]
    fn fuzz_delete() {
        let mut mem = [0; BST_MAX_SIZE * node_size::<i32>()];
        let mut rbt: Bst<usize, BST_MAX_SIZE> = Bst::new(&mut mem);
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
            assert!(rbt.insert(*num).is_ok());
        }

        // Delete all the numbers
        random_numbers.shuffle(&mut rng);
        while let Some(num) = random_numbers.pop() {
            match rbt.delete(num) {
                Ok(_) => (),
                Err(e) => assert!(false, "{:?}", e),
            }
        }

        assert_eq!(rbt.storage.length, 0);
    }
}
