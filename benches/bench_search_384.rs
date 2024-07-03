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

    // u64 nums (converted into 384bit)
    let nums = random_numbers::<u32>(0, 100_000);
    let nums = nums.into_iter().map(|x| x.into()).collect::<Vec<U384>>();

    // RBT 384bit
    let mut mem = [0; MAX_SIZE * rbt::node_size::<U384>()];
    let mut rbt: rbt::Rbt<U384, MAX_SIZE> = rbt::Rbt::new(&mut mem);

    for i in &nums {
        rbt.insert(*i).unwrap();
    }
    group.bench_with_input(BenchmarkId::new("rbt", "384bit"), &rbt, |b, rbt| {
        b.iter(|| {
            for i in &nums {
                rbt.search(i).unwrap();
            }
        })
    });

    // BST 384bit
    let mut mem = [0; MAX_SIZE * bst::node_size::<U384>()];
    let mut bst: bst::Bst<U384, MAX_SIZE> = bst::Bst::new(&mut mem);
    for i in &nums {
        bst.insert(*i).unwrap();
    }
    group.bench_with_input(BenchmarkId::new("bst", "384bit"), &bst, |b, bst| {
        b.iter(|| {
            for i in &nums {
                bst.search(i).unwrap();
            }
        })
    });

    // SORTED SLICE 384bit
    let mut mem = [0; MAX_SIZE * size_of::<U384>()];
    let mut ss: sorted_slice::SortedSlice<U384> = sorted_slice::SortedSlice::new(&mut mem);
    for i in &nums {
        ss.add(*i).unwrap();
    }
    group.bench_with_input(BenchmarkId::new("sorted_slice", "384bit"), &ss, |b, ss| {
        b.iter(|| {
            for i in &nums {
                ss.search_with_key(i).unwrap();
            }
        })
    });

    group.finish()
}

criterion_group!(benches, benchmark_search_function);
criterion_main!(benches);
