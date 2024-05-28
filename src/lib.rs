// #![no_std]
pub mod bst;
pub mod rbt;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    OutOfSpace,
}
