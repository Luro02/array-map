use core::hash::Hasher;

use array_map::ArrayMap;
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};

// TODO: https://github.com/siscia/benchmark-vector-vs-hashmap/blob/master/src/main.rs

const CAPACITY: usize = 3083;

// This hasher will always cause a collision
#[derive(Default)]
pub struct CollisionHasher;

impl Hasher for CollisionHasher {
    fn write(&mut self, _: &[u8]) {}

    fn finish(&self) -> u64 {
        0
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");
    group.sample_size(300);

    for elems in [CAPACITY, CAPACITY / 2, CAPACITY / 3, CAPACITY / 4] {
        group.throughput(Throughput::Elements(elems as u64));
        group.bench_with_input(
            // the name of the benchmark group
            BenchmarkId::from_parameter(format!("ArrayMap({}/{})", elems, CAPACITY)),
            // data passed to the benchmark function
            &elems,
            // benchmark function
            |b, &elems| {
                b.iter_batched(
                    || {
                        (
                            (0..elems).zip((0..elems).map(|i| i * 2 + 5)),
                            ArrayMap::<_, _, CAPACITY, _>::new(),
                            /* ArrayMap::<_, _, CAPACITY,
                             * _>::with_hasher(BuildHasherDefault::<CollisionHasher>::
                             * default()), */
                        )
                    },
                    |(mut iter, mut map)| {
                        let (k, v) = iter.next().unwrap();
                        map.insert(k, v).unwrap();
                    },
                    BatchSize::SmallInput,
                );
                /*
                b.iter_batched(
                    || {
                        ArrayMap::<_, _, CAPACITY, _>::with_hasher(BuildHasherDefault::<
                            CollisionHasher,
                        >::default(
                        ))
                    },
                    |mut map| {
                        for (k, v) in (0..elems).zip((0..elems).map(|i| i * 2 + 5)) {
                            map.insert(k, v).unwrap();
                        }
                    },
                    BatchSize::SmallInput,
                );*/
            },
        );
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
