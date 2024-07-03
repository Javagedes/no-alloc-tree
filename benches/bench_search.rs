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

fn benchmark_search_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("search");
    let nums = random_numbers::<u32>(0, 100_000);

    // RBT 32bit
    let mut mem = [0; MAX_SIZE * rbt::node_size::<u32>()];
    let mut rbt: rbt::Rbt<u32, MAX_SIZE> = rbt::Rbt::new(&mut mem);
    for i in &nums {
        rbt.insert(*i).unwrap();
    }
    group.bench_with_input(BenchmarkId::new("rbt", "32bit"), &rbt, |b, rbt| {
        b.iter(|| {
            for i in &nums {
                rbt.search(i).unwrap();
            }
        })
    });

    // BST 32bit
    let mut mem = [0; MAX_SIZE * bst::node_size::<u32>()];
    let mut bst: bst::Bst<u32, MAX_SIZE> = bst::Bst::new(&mut mem);
    for i in &nums {
        bst.insert(*i).unwrap();
    }
    group.bench_with_input(BenchmarkId::new("bst", "32bit"), &bst, |b, bst| {
        b.iter(|| {
            for i in &nums {
                bst.search(i).unwrap();
            }
        })
    });

    // SORTED SLICE 32bit
    let mut mem = [0; MAX_SIZE * size_of::<u32>()];
    let mut ss: sorted_slice::SortedSlice<u32> = sorted_slice::SortedSlice::new(&mut mem);
    for i in &nums {
        ss.add(*i).unwrap();
    }
    group.bench_with_input(BenchmarkId::new("sorted_slice", "32bit"), &ss, |b, ss| {
        b.iter(|| {
            for i in &nums {
                ss.search_with_key(i).unwrap();
            }
        })
    });

    // 128bit nums
    let nums = random_numbers::<i128>(0, 100_000);

    // RBT 128bit
    let mut mem = [0; MAX_SIZE * rbt::node_size::<i128>()];
    let mut rbt: rbt::Rbt<i128, MAX_SIZE> = rbt::Rbt::new(&mut mem);
    for i in &nums {
        rbt.insert(*i).unwrap();
    }
    group.bench_with_input(BenchmarkId::new("rbt", "128bit"), &rbt, |b, rbt| {
        b.iter(|| {
            for i in &nums {
                rbt.search(i).unwrap();
            }
        })
    });

    // BST 128bit
    let mut mem = [0; MAX_SIZE * bst::node_size::<i128>()];
    let mut bst: bst::Bst<i128, MAX_SIZE> = bst::Bst::new(&mut mem);
    for i in &nums {
        bst.insert(*i).unwrap();
    }
    group.bench_with_input(BenchmarkId::new("bst", "128bit"), &bst, |b, bst| {
        b.iter(|| {
            for i in &nums {
                bst.search(i).unwrap();
            }
        })
    });

    // SORTED SLICE 128bit
    let mut mem = [0; MAX_SIZE * size_of::<i128>()];
    let mut ss: sorted_slice::SortedSlice<i128> = sorted_slice::SortedSlice::new(&mut mem);
    for i in &nums {
        ss.add(*i).unwrap();
    }
    group.bench_with_input(BenchmarkId::new("sorted_slice", "128bit"), &ss, |b, ss| {
        b.iter(|| {
            for i in &nums {
                ss.search_with_key(i).unwrap();
            }
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_search_function);
criterion_main!(benches);
