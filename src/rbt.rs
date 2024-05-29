extern crate alloc;

use super::{Error, Result};
use core::mem::MaybeUninit;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

const RED: bool = false;
const BLACK: bool = true;

/// A on-stack storage container for the nodes of a red-black tree.
struct Storage<D, const SIZE: usize>
where
    D: PartialOrd,
{
    data: [(bool, MaybeUninit<Node<D>>); SIZE],
    length: usize,
}

impl<D, const SIZE: usize> Storage<D, {SIZE}>
where
    D: PartialOrd,
{
    /// Create a new storage container.
    fn new() -> Storage<D, SIZE> {
        Storage {
            data: unsafe { MaybeUninit::zeroed().assume_init() },
            length: 0,
        }
    }

    /// Create a new storage container at the given pointer.
    /// 
    /// # Safety
    /// 
    /// The pointer must be valid and must not be used after the storage is dropped.
    /// This function will zero out all memory at the given address for the storage.
    unsafe fn new_at(ptr: *mut Storage<D, SIZE>) -> &'static Storage<D, SIZE> {
            ptr::write(ptr, Storage {
                data: MaybeUninit::zeroed().assume_init(),
                length: 0,
            });
            &*ptr
    }

    /// Add a new node to the storage container, returning a mutable reference to the node.
    fn add(&mut self, data: D) -> Result<&mut Node<D>> {
        if let Some(index) = self.first_null() {
            self.data[index] = (true, MaybeUninit::new(Node::new(data)));
            self.length += 1;
            let (_, node) = self.data.get_mut(index).unwrap();
            return Ok(unsafe { node.assume_init_mut() });
        }
        Err(Error::OutOfSpace)
    }

    /// Delete a node from the storage container.
    fn delete(&mut self, ptr: *mut Node<D>) {
        // Calculate the index of the node in the storage container based off the pointer.
        let index = (ptr as usize - self.data.as_ptr() as usize) / core::mem::size_of::<(bool, MaybeUninit<Node<D>>)>();
        self.data[index].0 = false;
        self.length -= 1;
    }

    /// Find the first null node in the storage container.
    fn first_null(&self) -> Option<usize> {
        for (index, (init, _)) in self.data.iter().enumerate() {
            if !init {
                return Some(index);
            }
        }
        None
    }

    fn len(&self) -> usize {
        self.length
    }
}

/// A red-black tree that can hold up to `SIZE` nodes.
/// 
/// The tree is implemented using the [AtomicPtr] structure, so the target must support atomic operations.
/// The storage is allocated on the stack with [Self::new] or statically at any address using [Self::new_at].

pub struct Rbt<D, const SIZE: usize>
where
    D: PartialOrd,
{
    storage: Storage<D, SIZE>,
    head: AtomicPtr<Node<D>>,
}

impl<D, const SIZE: usize> Rbt<D, {SIZE}>
where
    D: PartialOrd + Copy + core::fmt::Debug,
{
    pub fn new() -> Rbt<D, SIZE> {
        Rbt {
            storage: Storage::new(),
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    fn head(&self) -> Option<&Node<D>> {
        let head_ptr = self.head.load(Ordering::SeqCst);
        if head_ptr.is_null() {
            return None;
        }
        Some(unsafe { & *head_ptr })
    }

    pub fn insert(&mut self, data: D) -> Result<()> {
        let node = self.storage.add(data).unwrap();

        if self.head.load(Ordering::SeqCst).is_null() {
            node.set_color(BLACK);
            self.head.store(node, Ordering::SeqCst);
            return Ok(());
        }

        let head = unsafe { &mut *self.head.load(Ordering::SeqCst) };

        Self::insert_node(head, node);
        Self::fixup(&self.head, node);
        head.set_color(BLACK);

        return Ok(());
    }

    pub fn search(&self, data: D) -> Option<D> {
        let mut current_idx = self.head();
        while let Some(node) = current_idx {
            if data == node.data {
                return Some(node.data);
            } else if data < node.data {
                current_idx = node.left();
            } else {
                current_idx = node.right();
            }
        }
        None
    }

    pub fn delete(&mut self, data: D) -> Result<()> {
        let Some(head) = self.head() else {
            return Err(Error::NotFound);
        };
        let mut current = head;
        loop {
            if data == current.data {
                break;
            } else if data < current.data {
                if let Some(left) = current.left() {
                    current = left;
                } else {
                    return Err(Error::NotFound);
                }
            } else {
                if let Some(right) = current.right() {
                    current = right;
                } else {
                    return Err(Error::NotFound);
                }
            }
        }
        
        let color = current.is_red();

        if current.left().is_none() | current.right().is_none() {
            Self::delete_simple(head, current);
            self.storage.delete(current.as_mut_ptr());
            return Ok(())
        } else {
            Self::delete_complex(current);
        };

        if color == BLACK {
            todo!() // maybe? Self::fixup(&self.head, new_node.unwrap());
        }

        todo!()
    }

    // Deletes a node with 0 or 1 children.
    fn delete_simple(head: &Node<D>, node: &Node<D>) {
        let parent = match node.parent() {
            Some(parent) => parent,
            None => head,
        };
        if let Some(left) = node.left() {
            left.set_parent(parent);
            if parent.left_ptr() == node.as_mut_ptr() {
                parent.set_left(left);
            } else {
                parent.set_right(left);
            }
        } else if let Some(right) = node.right() {
            right.set_parent(node);
            if parent.left_ptr() == node.as_mut_ptr() {
                parent.set_left(right);
            } else {
                parent.set_right(right);
            }
        } else {
            if parent.left_ptr() == node.as_mut_ptr() {
                parent.set_left(ptr::null_mut());
            } else {
                parent.set_right(ptr::null_mut());
            }
        }
    }

    // Deletes a node with 2 children.
    fn delete_complex(node: &Node<D>) {
        todo!()
    }

    fn insert_node(start: &Node<D>, node: &Node<D>) {
        let mut current = start;
        loop {
            if node.data < current.data {
                if let Some(left) = current.left() {
                    current = left;
                } else {
                    current.set_left(node);
                    node.set_parent(current);
                    break;
                }
            } else {
                if let Some(right) = current.right() {
                    current = right;
                } else {
                    current.set_right(node);
                    node.set_parent(current);
                    break;
                }
            }
        }
    }

    fn rotate_left(head: &AtomicPtr<Node<D>>, node: &Node<D>) {
        let right_child = node.right().expect("Right Child should always exist when rotating.");

        node.set_right(right_child.left_ptr());
        if let Some(left) = right_child.left() {
            left.set_parent(node);
        }

        right_child.set_left(node);
        node.set_parent(right_child);

        let parent = node.parent().unwrap();
        if Node::is_null(&node.parent) {
            head.store(node.parent.load(Ordering::SeqCst), Ordering::SeqCst);
        } else if parent.left_ptr() == node.as_mut_ptr() {
            parent.set_left(node);
            node.set_parent(parent);
        } else if parent.right_ptr() == node.as_mut_ptr() {
            parent.set_right(node);
            node.set_parent(parent);
        } else {
            panic!("Node is not a child of it's parents");
        }

        right_child.set_parent(parent);
    }

    // https://www.happycoders.eu/algorithms/red-black-tree-java/
    fn rotate_right(head: &AtomicPtr<Node<D>>, node: &Node<D>) {
        let left_child = node.left().unwrap();

        node.set_left(left_child.right_ptr());
        if let Some(right) = left_child.right() {
            right.set_parent(node);
        }

        left_child.set_right(node);
        node.set_parent(left_child);

        let parent = node.parent().unwrap();
        if Node::is_null(&node.parent) {
            head.store(node.parent.load(Ordering::SeqCst), Ordering::SeqCst);
        } else if parent.left.load(Ordering::SeqCst) == node.as_mut_ptr() {
            parent.set_left(node);
            node.set_parent(parent);
        } else if parent.right.load(Ordering::SeqCst) == node.as_mut_ptr() {
            parent.set_right(node);
            node.set_parent(parent);
        } else {
            panic!("Node is not a child of it's parents");
        }

        left_child.set_parent(parent);
    }

    fn fixup(head: &AtomicPtr<Node<D>>, node: &Node<D>) {
        // Case 1: The node is the root of the tree, no fixups needed.
        let Some(parent) = node.parent() else {
            node.set_color(BLACK);
            return;
        };

        // The parent is black, no fixups needed.
        if parent.is_black() {
            return;
        }

        // Case 2 is enforced by setting the parent to black. If the parent is red, the grandparent should exist.
        let grandparent = parent.parent().expect("Parent is red, grandparent should exist");
        let uncle = Node::sibling(parent);
        
        // Case 3: Uncle is red, recolor parent, grandparent, uncle
        if let Some(uncle) = uncle && uncle.is_red() {
            parent.set_color(BLACK);
            grandparent.set_color(RED);
            uncle.set_color(BLACK);

            todo!(); // Move to grandparent and do this same logic again. Need a while loop
        }

        // Parent is left child of grandparent
        else if parent.as_mut_ptr() == grandparent.left.load(Ordering::SeqCst) {
            // Case 4a: uncle is black and node is left->right "inner child" of it's grandparent
            if node.as_mut_ptr() == parent.right.load(Ordering::SeqCst) {
                Self::rotate_left(head, parent);
                grandparent.set_left(node);
            }
            // Case 5a: uncle is black and node is left->left "outer child" of it's grandparent
            Self::rotate_right(head, grandparent); //todo, need updated parent??
            node.set_color(BLACK);
            grandparent.set_color(RED);
        }
        // Parent is right child of grandparent
        else if parent.as_mut_ptr() == grandparent.right.load(Ordering::SeqCst) {
            // Case 4b: unclde is black and node is right->left "inner child" of its grandparent
            if node.as_mut_ptr() == parent.left.load(Ordering::SeqCst) {
                Self::rotate_right(head, parent);
                grandparent.set_right(node);
            }
            Self::rotate_left(head, grandparent);

            node.set_color(BLACK);
            grandparent.set_color(RED);
        }
        panic!("Parent is not a child of grandparent")
    }

    fn dfs(&self, node: Option<&Node<D>>, values: &mut alloc::vec::Vec<D>) {
        if let Some(node) = node {
            self.dfs(node.left(), values);
            values.push(node.data);
            self.dfs(node.right(), values);
        }
    }
}

#[derive(Debug)]
struct Node<D>
where
    D: PartialOrd,
{
    data: D,
    color: AtomicBool,
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
            color: AtomicBool::new(RED),
            parent: AtomicPtr::default(),
            left: AtomicPtr::default(),
            right: AtomicPtr::default(),
        }
    }

    fn set_color(&self, color: bool) {
        self.color.store(color, Ordering::SeqCst);
    }

    fn is_red(&self) -> bool {
        self.color.load(Ordering::SeqCst) == RED
    }

    fn is_black(&self) -> bool {
        self.color.load(Ordering::SeqCst) == BLACK
    }

    #[inline(always)]
    /// Used when you care whether or not the node is null.
    fn right(&self) -> Option<&Node<D>> {
        let node = self.right.load(Ordering::SeqCst);
        if node.is_null() {
            return None;
        }
        Some(unsafe { & *node })
    }

    /// Used when you don't care whether or not the node is null.
    #[inline(always)]
    fn right_ptr(&self) -> *mut Node<D> {
        self.right.load(Ordering::SeqCst)
    }

    #[inline(always)]
    fn set_right<N: Into<*mut Node<D>>>(&self, node: N)
    {
        self.right.store(node.into(), Ordering::SeqCst);
    }

    #[inline(always)]
    fn left(&self) -> Option<&Node<D>> {
        let node = self.left.load(Ordering::SeqCst);
        if node.is_null() {
            return None;
        }
        Some(unsafe { & *node })
    }

    fn left_ptr(&self) -> *mut Node<D> {
        self.left.load(Ordering::SeqCst)
    }

    #[inline(always)]
    fn set_left<N: Into<*mut Node<D>>>(&self, node: N)
    {
        self.left.store(node.into(), Ordering::SeqCst);
    }

    fn parent(&self) -> Option<&Node<D>> {
        let node = self.parent.load(Ordering::SeqCst);
        if node.is_null() {
            return None;
        }
        Some(unsafe { &*node })
    }

    fn set_parent<N: Into<*mut Node<D>>>(&self, node: N) {
        self.parent.store(node.into(), Ordering::SeqCst);
    }

    #[inline(always)]
    fn as_mut_ptr(&self) -> *mut Node<D> {
        self as *const _ as *mut _
    }

    #[inline(always)]
    fn is_null(node: &AtomicPtr<Node<D>>) -> bool {
        node.load(Ordering::SeqCst).is_null()
    }

    fn sibling(node: &Node<D>) -> Option<&Node<D>> {
        let parent = node.parent()?;
        match node.as_mut_ptr() {
            ptr if ptr == parent.left_ptr() => parent.right(),
            ptr if ptr == parent.right_ptr() => parent.left(),
            _ => panic!("Node is not a child of its parent."),
        }
    }
}

impl <D>From<&Node<D>> for *mut Node<D>
where
    D: PartialOrd,
{
    fn from(node: &Node<D>) -> *mut Node<D> {
        node.as_mut_ptr()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::{Node, Rbt};
    use core::{ptr::null_mut, sync::atomic::{AtomicPtr, Ordering}};
    use std::println;

    const RBT_MAX_SIZE: usize = 0x1000;

    #[test]
    fn simple_test() {
            let mut rbt: Rbt<i32, RBT_MAX_SIZE> = Rbt::new();
            assert!(rbt.insert(5).is_ok());
            assert_eq!(rbt.storage.length, 1);
            assert!(rbt.insert(3).is_ok());
            assert!(rbt.insert(7).is_ok());
            assert!(rbt.insert(2).is_ok());
            assert!(rbt.insert(6).is_ok());
            assert!(rbt.insert(8).is_ok());
            assert!(rbt.insert(9).is_ok());
            assert!(rbt.insert(10).is_ok());

            let mut values = std::vec::Vec::new();
            rbt.dfs(rbt.head(), &mut values);
            println!("{:?}", values);

            for (initialized, node) in rbt.storage.data.iter() {
                if *initialized {
                    println!("{:?}", unsafe { node.assume_init_ref() });
                }
            }
        }

    #[test]
    fn test_rotate_right() {
        /* Verifies that the rotate right function works as expected.
              [50]              [75]
              /  \              /  \
            [10][75]    <--   [50][85]
                /  \          /  \
              [70][85]      [10][70]
         */
        let node = Node::new(75);
        let left = Node::new(50);
        let right = Node::new(85);
        let left_l = Node::new(10);
        let left_r = Node::new(70);

        left.set_left(&left_l);
        left_l.set_parent(&left);
        left.set_right(&left_r);
        left_r.set_parent(&left);
        node.set_left(&left);
        left.set_parent(&node);
        node.set_right(&right);
        right.set_parent(&node);

        let head = AtomicPtr::<Node<i32>>::default();

        Rbt::<i32, RBT_MAX_SIZE>::rotate_right(&head, &node);
        
        // Check left[50] <-> left_l[10] connection
        assert_eq!(left.left().unwrap().as_mut_ptr(), left_l.as_mut_ptr());
        assert_eq!(left_l.parent().unwrap().as_mut_ptr(), left.as_mut_ptr());

        // check left[50] <-> left_r[70] connection 
        assert_eq!(left.right().unwrap().as_mut_ptr(), node.as_mut_ptr());
        assert_eq!(node.parent().unwrap().as_mut_ptr(), left.as_mut_ptr());

        // check left_l[10] has no children
        assert!(left_l.left().is_none());
        assert!(left_l.right().is_none());

        // check node[75] <-> left_r[70] connection
        assert_eq!(node.left().unwrap().as_mut_ptr(), left_r.as_mut_ptr());
        assert_eq!(left_r.parent().unwrap().as_mut_ptr(), node.as_mut_ptr());

        // check node[75] <-> right[85] connection
        assert_eq!(node.right().unwrap().as_mut_ptr(), right.as_mut_ptr());
        assert_eq!(right.parent().unwrap().as_mut_ptr(), node.as_mut_ptr());

        // Check right_r[70] has no children
        assert!(left_r.left().is_none());
        assert!(left_r.right().is_none());

        // Check right[85] has no children
        assert!(right.left().is_none());
        assert!(right.right().is_none());
    }

    #[test]
    fn test_rotate_left() {
        /* Verifies that the rotate left function works as expected.
              [50]              [75]
              /  \              /  \
            [10][75]    -->   [50][85]
                /  \          /  \
              [70][85]      [10][70]
         */
        let node = Node::new(50);
        let left = Node::new(10);
        let right = Node::new(75);
        let right_l = Node::new(70);
        let right_r = Node::new(85);

        right.set_left(&right_l);
        right_l.set_parent(&right);
        right.set_right(&right_r);
        right_r.set_parent(&right);
        node.set_left(&left);
        left.set_parent(&node);
        node.set_right(&right);
        right.set_parent(&node);

        let head = AtomicPtr::<Node<i32>>::default();

        Rbt::<i32, RBT_MAX_SIZE>::rotate_left(&head, &node);

        // Check right[75] <-left-> node[50] connection
        assert_eq!(right.left().unwrap().as_mut_ptr(), node.as_mut_ptr());
        assert_eq!(node.parent().unwrap().as_mut_ptr(), right.as_mut_ptr());

        // Check right[75] <-right-> right_r[85] connection
        assert_eq!(right.right().unwrap().as_mut_ptr(), right_r.as_mut_ptr());
        assert_eq!(right_r.parent().unwrap().as_mut_ptr(), right.as_mut_ptr());

        // Check node[50] <-left-> left[10] connection
        assert_eq!(node.left().unwrap().as_mut_ptr(), left.as_mut_ptr());
        assert_eq!(left.parent().unwrap().as_mut_ptr(), node.as_mut_ptr());

        // Check node[50] <-right-> right_l[70] connection
        assert_eq!(node.right().unwrap().as_mut_ptr(), right_l.as_mut_ptr());
        assert_eq!(right_l.parent().unwrap().as_mut_ptr(), node.as_mut_ptr());
        
        // Check left[10] has no children
        assert!(left.left().is_none());
        assert!(left.right().is_none());
        
        // Check right_r[85] has no children
        assert!(right_r.left().is_none());
        assert!(right_r.right().is_none());
        
        // Check right_l[70] has no children
        assert!(right_l.left().is_none());
        assert!(right_l.right().is_none());
    }

    #[test]
    fn test_delete_from_storage() {
        let mut rbt = Rbt::<i32, 10>::new();
        rbt.insert(5).unwrap();
        rbt.insert(3).unwrap();
        assert_eq!(rbt.storage.len(), 2);
        assert_eq!(rbt.storage.data.iter().filter(|(i, _)|{*i}).count(), 2);
        rbt.delete(5).unwrap();
        assert_eq!(rbt.storage.len(), 1);
        assert_eq!(rbt.storage.data.iter().filter(|(i, _)|{*i}).count(), 1);
        rbt.delete(3).unwrap();
        assert_eq!(rbt.storage.len(), 0);
        assert_eq!(rbt.storage.data.iter().filter(|(i, _)|{*i}).count(), 0);
    }
    #[test]
    fn test_delete_simple() {
        /* Verifies that deleting a node with a single child or no child works as expected.
                [50]      [50]
                /          /
              [10]   ->  [05]   ->   [50]
               /
             [05]
        */
        let node = Node::new(50);
        let left = Node::new(10);
        let left_l = Node::new(5);

        node.set_left(&left);
        left.set_parent(&node);
        left.set_left(&left_l);
        left_l.set_parent(&left);
        
        // Delete a node with a single child.
        Rbt::<i32, RBT_MAX_SIZE>::delete_simple(&node, &left);
        assert_eq!(node.left().unwrap().as_mut_ptr(), left_l.as_mut_ptr());
        assert_eq!(left_l.parent().unwrap().as_mut_ptr(), node.as_mut_ptr());

        // Delete a node with no children.
        Rbt::<i32, RBT_MAX_SIZE>::delete_simple(&node, &left_l);
        assert!(node.left().is_none());
    }

}

#[cfg(test)]
mod fuzz_tests {
    extern crate std;
    use super::{Node, Rbt};
    use core::sync::atomic::AtomicPtr;
    use rand::seq::SliceRandom;
    use rand::Rng;
    use std::collections::HashSet;
    use std::vec::Vec;

    const RBT_MAX_SIZE: usize = 0x1000;

    #[test]
    fn fuzz_insert() {
        for _ in 0..100 {
            let mut rbt: Rbt<usize, RBT_MAX_SIZE> = Rbt::new();
            let mut rng = rand::thread_rng();
            let min = 1;
            let max = 100_000;

            let mut random_numbers = HashSet::new();

            while random_numbers.len() < RBT_MAX_SIZE - 1 {
                let num = rng.gen_range(min..=max);
                random_numbers.insert(num);
            }

            let mut random_numbers: Vec<_> = random_numbers.into_iter().collect();
            random_numbers.shuffle(&mut rng);

            assert_eq!(random_numbers.len(), RBT_MAX_SIZE - 1);
            for num in random_numbers.iter() {
                assert!(rbt.insert(*num).is_ok());
            }

            random_numbers.sort();

            let mut ordered_numbers = Vec::new();
            rbt.dfs(rbt.head(), &mut ordered_numbers);
            assert_eq!(ordered_numbers, random_numbers);
        }
    }

    #[test]
    fn fuzz_delete() {
        let mut rbt: Rbt<usize, RBT_MAX_SIZE> = Rbt::new();
        let mut rng = rand::thread_rng();
        let min = 1;
        let max = 100_000;

        let mut random_numbers = HashSet::new();
        while random_numbers.len() < RBT_MAX_SIZE {
            let num = rng.gen_range(min..=max);
            random_numbers.insert(num);
        }

        let mut random_numbers: Vec<_> = random_numbers.into_iter().collect();
        random_numbers.shuffle(&mut rng);
        
        assert_eq!(random_numbers.len(), RBT_MAX_SIZE);
        for num in random_numbers.iter() {
            assert!(rbt.insert(*num).is_ok());
        }

        // Delete all the numbers
        random_numbers.shuffle(&mut rng);
        while let Some(num) = random_numbers.pop() {
            assert!(rbt.delete(num).is_ok());
        }
    }
    
    #[test]
    fn fuzz_search() {
        let mut bst: Rbt<usize, RBT_MAX_SIZE> = Rbt::new();
        let mut rng = rand::thread_rng();
        let min = 1;
        let max = 100_000;

        let mut random_numbers = HashSet::new();
        while random_numbers.len() < RBT_MAX_SIZE {
            let num = rng.gen_range(min..=max);
            random_numbers.insert(num);
        }

        let mut random_numbers: Vec<_> = random_numbers.into_iter().collect();
        random_numbers.shuffle(&mut rng);

        assert_eq!(random_numbers.len(), RBT_MAX_SIZE);
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
                rng.gen_range(0..=min - 1)
            } else {
                rng.gen_range(max + 1..=max + 50_000)
            };
            assert!(bst.search(random_number).is_none());
        }
    }
}
