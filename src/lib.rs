#![no_std]
#![feature(let_chains)]
#![feature(is_sorted)]
pub mod bst;
pub mod rbt;
pub mod sorted_slice;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    OutOfSpace,
    NotFound,
    AlreadyExists,
}

pub trait SortedSliceKey {
    type Key: Ord;
    fn ordering_key(&self) -> &Self::Key;
  }
