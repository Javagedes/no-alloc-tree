use super::{Error, Result};
use arrayvec::ArrayVec;
use core::cell::Cell;
use core::mem::MaybeUninit;
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};

const RED: bool = false;
const BLACK: bool = true;
const RBT_MAX_SIZE: usize = 4096;

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
            data: array_init::array_init(|_| (false, MaybeUninit::uninit())),
            length: 0,
        }
    }

    fn add(&mut self, data: D) -> Result<*mut Node<D>> {
        if let Some(index) = self.first_null() {
            self.data[index] = (true, MaybeUninit::new(Node::new(data)));
            self.length += 1;
            let (_, node) = self.data.get_mut(index).unwrap();
            let ptr = node.as_mut_ptr();
            return Ok(ptr);
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
    D: PartialOrd,
{
    fn new() -> Rbt<D> {
        Rbt {
            storage: Storage::new(),
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    fn insert(&mut self, data: D) -> Result<()> {
        let ptr = self.storage.add(data).unwrap();
        if self.head.load(Ordering::SeqCst).is_null() {
            self.head.store(ptr, Ordering::SeqCst);
            return Ok(());
        }

        let raw_ptr = self.head.load(Ordering::SeqCst);
        let node = unsafe { &*raw_ptr };

        node.left.store(ptr, Ordering::SeqCst);
        node.set_right(node.left());
        //     (*raw_ptr)
        //         .left
        //         .store(ptr, Ordering::SeqCst);

        // }


        return Ok(());
        //todo!()
    }
}

#[derive(Debug)]
struct Node<D>
where
    D: PartialOrd,
{
    data: D,
    color: bool,
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
            color: RED,
            left: AtomicPtr::default(),
            right: AtomicPtr::default(),
        }
    }

    fn right(&self) -> &mut Node<D> {
        let node = self.right.load(Ordering::SeqCst);
        unsafe { &mut *node }
    }

    fn set_right(&self, node: &Node<D>) {
        self.right.store(node.as_mut_ptr(), Ordering::SeqCst);
    }

    fn left(&self) -> &mut Node<D> {
        let node = self.left.load(Ordering::SeqCst);
        unsafe { &mut *node }
    }

    fn set_left(&self, node: &Node<D>) {
        self.left.store(node.as_mut_ptr(), Ordering::SeqCst);
    }

    fn as_mut_ptr(&self) -> *mut Node<D> {
        self as *const _ as *mut _
    }
}

impl <D>AsMut<D> for Node<D>
where
    D: PartialOrd 
{
    fn as_mut(&mut self) -> &mut D {
        &mut self.data
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::Rbt;
    use std::println;

    #[test]
    fn simple_test() {
        let mut rbt = Rbt::new();
        assert!(rbt.insert(5).is_ok());
        assert_eq!(rbt.storage.length, 1);
        assert!(rbt.insert(3).is_ok());
        for (init, data) in rbt.storage.data.iter() {
            if *init {
                println!("{:?}", unsafe { data.assume_init_ref() });
                //let x_ptr = &x as *const i32;
                //println!("Address of x: {:?}", x_ptr)
            }
        }
    }
}
