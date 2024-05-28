extern crate alloc;

use super::{Error, Result};
use core::mem::MaybeUninit;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

const RED: bool = false;
const BLACK: bool = true;
const RBT_MAX_SIZE: usize = 0x1000;

struct Storage<D>
where
    D: PartialOrd,
{
    data: [(bool, MaybeUninit<Node<D>>); RBT_MAX_SIZE],
    length: usize,
}

impl<D> Storage<D>
where
    D: PartialOrd,
{
    fn new() -> Storage<D> {
        Storage {
            data: unsafe { MaybeUninit::zeroed().assume_init() },
            length: 0,
        }
    }

    fn add(&mut self, data: D) -> Result<&mut Node<D>> {
        if let Some(index) = self.first_null() {
            self.data[index] = (true, MaybeUninit::new(Node::new(data)));
            self.length += 1;
            let (_, node) = self.data.get_mut(index).unwrap();
            return Ok(unsafe { node.assume_init_mut() });
        }
        Err(Error::OutOfSpace)
    }

    fn first_null(&self) -> Option<usize> {
        for (index, (init, _)) in self.data.iter().enumerate() {
            if !init {
                return Some(index);
            }
        }
        None
    }
}

struct Rbt<D>
where
    D: PartialOrd,
{
    storage: Storage<D>,
    head: AtomicPtr<Node<D>>,
}

impl<D> Rbt<D>
where
    D: PartialOrd + Copy + core::fmt::Debug,
{
    fn new() -> Rbt<D> {
        Rbt {
            storage: Storage::new(),
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    fn head(&self) -> Option<&mut Node<D>> {
        let head_ptr = self.head.load(Ordering::SeqCst);
        if head_ptr.is_null() {
            return None;
        }
        Some(unsafe { &mut *head_ptr })
    }

    fn add_node(&mut self, data: D) -> Result<&mut Node<D>> {
        self.storage.add(data)
    }

    fn insert(&mut self, data: D) -> Result<()> {
        let node = self.storage.add(data).unwrap();

        if self.head.load(Ordering::SeqCst).is_null() {
            node.set_color(BLACK);
            self.head.store(node.as_mut_ptr(), Ordering::SeqCst);
            return Ok(());
        }

        let head = unsafe { &mut *self.head.load(Ordering::SeqCst) };

        Self::insert_node(head, node);
        //Self::fixup(&self.head, node);
        head.set_color(BLACK);

        return Ok(());
    }

    fn insert_node(start: &Node<D>, node: &Node<D>) {
        let mut current = start;
        loop {
            if node.data < current.data {
                if let Some(left) = current.left() {
                    current = left;
                } else {
                    current.set_left(node);
                    break;
                }
            } else {
                if let Some(right) = current.right() {
                    current = right;
                } else {
                    current.set_right(node);
                    break;
                }
            }
        }
    }

    fn rotate_left(head: &AtomicPtr<Node<D>>, node: &Node<D>) {
        let right_child = node.right().unwrap();

        node.set_right(right_child.left().unwrap());
        if let Some(left) = right_child.left() {
            left.set_parent(node);
        }

        right_child.set_left(node);
        node.set_parent(right_child);

        let parent = node.parent().unwrap();
        if Node::is_null(&node.parent) {
            head.store(node.parent.load(Ordering::SeqCst), Ordering::SeqCst);
        } else if parent.left.load(Ordering::SeqCst) == node.as_mut_ptr() {
            parent.set_left(node);
        } else if parent.right.load(Ordering::SeqCst) == node.as_mut_ptr() {
            parent.set_right(node);
        } else {
            panic!("Node is not a child of it's parents");
        }

        right_child.set_parent(parent);
    }

    // https://www.happycoders.eu/algorithms/red-black-tree-java/
    fn rotate_right(head: &AtomicPtr<Node<D>>, node: &Node<D>) {
        let left_child = node.left().unwrap();

        node.set_left(left_child.right().unwrap());
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
        } else if parent.right.load(Ordering::SeqCst) == node.as_mut_ptr() {
            parent.set_right(node);
        } else {
            panic!("Node is not a child of it's parents");
        }

        left_child.set_parent(parent);
    }

    fn fixup(head: &AtomicPtr<Node<D>>, node: &Node<D>) {
        todo!()
    }

    fn dfs(&self, node: Option<&mut Node<D>>, values: &mut alloc::vec::Vec<D>) {
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
    fn right(&self) -> Option<&mut Node<D>> {
        let node = self.right.load(Ordering::SeqCst);
        if node.is_null() {
            return None;
        }
        Some(unsafe { &mut *node })
    }

    #[inline(always)]
    fn set_right(&self, node: &Node<D>) {
        node.set_parent(self);
        self.right.store(node.as_mut_ptr(), Ordering::SeqCst);
    }

    #[inline(always)]
    fn left(&self) -> Option<&mut Node<D>> {
        let node = self.left.load(Ordering::SeqCst);
        if node.is_null() {
            return None;
        }
        Some(unsafe { &mut *node })
    }

    #[inline(always)]
    fn set_left(&self, node: &Node<D>) {
        node.set_parent(self);
        self.left.store(node.as_mut_ptr(), Ordering::SeqCst);
    }

    fn parent(&self) -> Option<&mut Node<D>> {
        let node = self.parent.load(Ordering::SeqCst);
        if node.is_null() {
            return None;
        }
        Some(unsafe { &mut *node })
    }

    fn set_parent(&self, node: &Node<D>) {
        self.parent.store(node.as_mut_ptr(), Ordering::SeqCst);
    }

    #[inline(always)]
    fn as_mut_ptr(&self) -> *mut Node<D> {
        self as *const _ as *mut _
    }

    #[inline(always)]
    fn is_null(node: &AtomicPtr<Node<D>>) -> bool {
        node.load(Ordering::SeqCst).is_null()
    }
}

impl<D> AsMut<D> for Node<D>
where
    D: PartialOrd,
{
    fn as_mut(&mut self) -> &mut D {
        &mut self.data
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::{Node, Rbt};
    use std::println;

    //    #[test]
    //     fn simple_test() {
    //         let mut rbt = Rbt::new();
    //         assert!(rbt.insert(5).is_ok());
    //         assert_eq!(rbt.storage.length, 1);
    //         assert!(rbt.insert(3).is_ok());
    //         assert!(rbt.insert(7).is_ok());
    //         assert!(rbt.insert(2).is_ok());
    //         assert!(rbt.insert(6).is_ok());
    //         assert!(rbt.insert(8).is_ok());
    //         assert!(rbt.insert(9).is_ok());
    //         assert!(rbt.insert(10).is_ok());

    //         let mut values = std::vec::Vec::new();
    //         rbt.dfs(rbt.head(), &mut values);
    //         println!("{:?}", values);

    //         for (initialized, node) in rbt.storage.data.iter() {
    //             if *initialized {
    //                 println!("{:?}", unsafe { node.assume_init_ref() });
    //             }
    //         }
    //     }
}

#[cfg(test)]
mod fuzz_tests {
    extern crate std;
    use super::RBT_MAX_SIZE;
    use super::{Node, Rbt};
    use core::sync::atomic::AtomicPtr;
    use rand::seq::SliceRandom;
    use rand::Rng;
    use std::collections::HashSet;
    use std::vec::Vec;

    #[test]
    fn fuzz_insert() {
        for _ in 0..1 {
            let mut rbt = Rbt::<usize>::new();
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
                std::println!("{:?}", num);
                assert!(rbt.insert(*num).is_ok());
            }

            random_numbers.sort();

            let mut ordered_numbers = Vec::new();
            rbt.dfs(rbt.head(), &mut ordered_numbers);
            assert_eq!(ordered_numbers, random_numbers);
            // for (initialized, node) in rbt.storage.data.iter() {
            //     if *initialized {
            //         std::println!("{:?}", unsafe { node.assume_init_ref() });
            //     }
            // }
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
        left.set_right(&left_r);
        node.set_left(&left);
        node.set_right(&right);

        let head = AtomicPtr::<Node<i32>>::default();

        Rbt::rotate_right(&head, &node);

        assert_eq!(left.left().unwrap().as_mut_ptr(), left_l.as_mut_ptr());
        assert_eq!(left.right().unwrap().as_mut_ptr(), node.as_mut_ptr());
        assert!(left_l.left().is_none());
        assert!(left_l.right().is_none());
        assert_eq!(node.left().unwrap().as_mut_ptr(), left_r.as_mut_ptr());
        assert_eq!(node.right().unwrap().as_mut_ptr(), right.as_mut_ptr());
        assert!(left_r.left().is_none());
        assert!(left_r.right().is_none());
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
        right.set_right(&right_r);
        node.set_left(&left);
        node.set_right(&right);

        let head = AtomicPtr::<Node<i32>>::default();

        Rbt::rotate_left(&head, &node);

        assert_eq!(right.left().unwrap().as_mut_ptr(), node.as_mut_ptr());
        assert_eq!(node.left().unwrap().as_mut_ptr(), left.as_mut_ptr());
        assert!(left.left().is_none());
        assert!(left.right().is_none());
        assert_eq!(right.right().unwrap().as_mut_ptr(), right_r.as_mut_ptr());
        assert!(right_r.left().is_none());
        assert!(right_r.right().is_none());
        assert_eq!(node.right().unwrap().as_mut_ptr(), right_l.as_mut_ptr());
        assert!(right_l.left().is_none());
        assert!(right_l.right().is_none());
    }
}
