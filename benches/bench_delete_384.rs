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

static mut MEM_U384: [u8; 327680] = [0; MAX_SIZE * bst::node_size::<U384>()];

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

fn benchmark_delete_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("delete");

    // u64 nums (converted into 384bit)
    let nums = random_numbers::<u32>(0, 100_000);
    let nums = nums.into_iter().map(|x| x.into()).collect::<Vec<U384>>();
    let mut nums_shuffled = nums.clone();
    nums_shuffled.shuffle(&mut rand::thread_rng());

    // RBT 384bit
    // group.bench_function(
    //     BenchmarkId::new("rbt", "384bit"), |b| {
    //     b.iter_batched_ref(
    //         || {
    //             let mut rbt: rbt::Rbt<U384, MAX_SIZE> = rbt::Rbt::new(unsafe {&mut MEM_U384});
    //             for i in &nums {
    //                 rbt.insert(*i).unwrap();
    //             }
    //             rbt
    //         }, |rbt|{
    //             for i in &nums {
    //                 rbt.delete(*i).unwrap();
    //             }
    //         },
    //         criterion::BatchSize::PerIteration
    //     );
    // });

    // // BST 384bit
    group.bench_function(BenchmarkId::new("bst", "384bit"), |b| {
        b.iter_batched_ref(
            || {
                let mut bst: bst::Bst<U384, MAX_SIZE> = bst::Bst::new(unsafe { &mut MEM_U384 });
                for i in &nums {
                    bst.insert(*i).unwrap();
                }
                bst
            },
            |bst| {
                for i in &nums_shuffled {
                    bst.delete(*i).unwrap();
                }
            },
            criterion::BatchSize::PerIteration,
        );
    });

    // SORTED SLICE 384bit
    group.bench_function(BenchmarkId::new("sorted_slice", "384bit"), |b| {
        b.iter_batched_ref(
            || {
                let mut ss: sorted_slice::SortedSlice<U384> =
                    sorted_slice::SortedSlice::new(unsafe { &mut MEM_U384 });
                for i in &nums {
                    ss.add(*i).unwrap();
                }
                ss
            },
            |ss| {
                for i in &nums_shuffled {
                    let idx = ss.search_idx_with_key(i).unwrap();
                    ss.remove_at_idx(idx).unwrap();
                }
            },
            criterion::BatchSize::PerIteration,
        );
    });

    group.finish()
}

criterion_group!(benches, benchmark_delete_function);
criterion_main!(benches);
