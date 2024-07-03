use alloc_tree::bst::BstKey;
use alloc_tree::sorted_slice::SortedSliceKey;
use alloc_tree::{bst, rbt, sorted_slice};
use core::num;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::{hash_set::IntoIter, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::mem::size_of;
use uint::construct_uint;

const MAX_SIZE: usize = 4096;

/// The size of MemorySpaceDescriptor
construct_uint! {
    pub struct U384(6);
}

fn random_numbers<D>(min: D, max: D) -> Vec<D>
where
    D: Copy + Eq + std::cmp::PartialOrd + Hash + rand::distributions::uniform::SampleUniform,
{
    let mut rng = rand::thread_rng();
    let mut nums: HashSet<D> = HashSet::new();
    while nums.len() < MAX_SIZE {
        let num: D = rng.gen_range(min..=max);
        nums.insert(num);
    }
    nums.into_iter().collect()
}

pub fn benchmark_insert_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");
    let nums = random_numbers::<u32>(0, 100_000);
    group.bench_with_input(BenchmarkId::new("rbt", "32bit"), &nums, |b, nums| {
        b.iter(|| {
            let mut mem = [0; MAX_SIZE * rbt::node_size::<u32>()];
            let mut rbt: rbt::Rbt<u32, MAX_SIZE> = rbt::Rbt::new(&mut mem);

            for i in nums {
                rbt.insert(*i).unwrap();
            }
        })
    });

    group.bench_with_input(BenchmarkId::new("bst", "32bit"), &nums, |b, nums| {
        b.iter(|| {
            let mut mem = [0; MAX_SIZE * bst::node_size::<u32>()];
            let mut bst: bst::Bst<u32, MAX_SIZE> = bst::Bst::new(&mut mem);

            for i in nums {
                bst.insert(*i).unwrap();
            }
        })
    });

    group.bench_with_input(
        BenchmarkId::new("sorted_slice", "32bit"),
        &nums,
        |b, nums| {
            b.iter(|| {
                let mut mem = [0; MAX_SIZE * size_of::<u32>()];
                let mut ss: sorted_slice::SortedSlice<u32> =
                    sorted_slice::SortedSlice::new(&mut mem);

                for i in nums {
                    ss.add(*i).unwrap();
                }
            })
        },
    );

    let nums = random_numbers::<i128>(0, 100_000);

    group.bench_with_input(BenchmarkId::new("rbt", "128bit"), &nums, |b, nums| {
        b.iter(|| {
            let mut mem = [0; MAX_SIZE * rbt::node_size::<i128>()];
            let mut rbt: rbt::Rbt<i128, MAX_SIZE> = rbt::Rbt::new(&mut mem);

            for i in nums {
                rbt.insert(*i).unwrap();
            }
        })
    });

    group.bench_with_input(BenchmarkId::new("bst", "128bit"), &nums, |b, nums| {
        b.iter(|| {
            let mut mem = [0; MAX_SIZE * bst::node_size::<i128>()];
            let mut bst: bst::Bst<i128, MAX_SIZE> = bst::Bst::new(&mut mem);

            for i in nums {
                bst.insert(*i).unwrap();
            }
        })
    });

    group.bench_with_input(
        BenchmarkId::new("sorted_slice", "128bit"),
        &nums,
        |b, nums| {
            b.iter(|| {
                let mut mem = [0; MAX_SIZE * size_of::<i128>()];
                let mut ss: sorted_slice::SortedSlice<i128> =
                    sorted_slice::SortedSlice::new(&mut mem);

                for i in nums {
                    ss.add(*i).unwrap();
                }
            })
        },
    );

    let nums = random_numbers::<u32>(0, 100_000);

    group.bench_with_input(BenchmarkId::new("rbt", "384bit"), &nums, |b, nums| {
        b.iter(|| {
            let mut mem = [0; MAX_SIZE * rbt::node_size::<U384>()];
            let mut rbt: rbt::Rbt<U384, MAX_SIZE> = rbt::Rbt::new(&mut mem);

            for i in nums {
                rbt.insert((*i).into()).unwrap();
            }
        })
    });

    group.bench_with_input(BenchmarkId::new("bst", "384bit"), &nums, |b, nums| {
        b.iter(|| {
            let mut mem = [0; MAX_SIZE * bst::node_size::<U384>()];
            let mut bst: bst::Bst<U384, MAX_SIZE> = bst::Bst::new(&mut mem);

            for i in nums {
                bst.insert((*i).into()).unwrap();
            }
        })
    });

    group.bench_with_input(
        BenchmarkId::new("sorted_slice", "384bit"),
        &nums,
        |b, nums| {
            b.iter(|| {
                let mut mem = [0; MAX_SIZE * size_of::<U384>()];
                let mut ss: sorted_slice::SortedSlice<U384> =
                    sorted_slice::SortedSlice::new(&mut mem);

                for i in nums {
                    ss.add((*i).into()).unwrap();
                }
            })
        },
    );

    group.finish();
}

criterion_group!(benches, benchmark_insert_function);
criterion_main!(benches);
