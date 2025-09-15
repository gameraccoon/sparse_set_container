# Sparse Set Container
A container based on a sparse set.

It is useful if you want a container with performance close to Vec but also to safely store the indexes to the elements (so that they are not invalidated on removals).  
E.g. you have a list of elements in UI that the user can add and remove, but you want to refer to the elements of that list from somewhere else.

[![crates.io][crates.io shield]][crates.io link]
[![Documentation][docs.rs badge]][docs.rs link]

[![Download Status][shields.io download count]][crates.io link]

<!--badge links start-->
[crates.io shield]: https://img.shields.io/crates/v/sparse_set_container?label=latest
[crates.io link]: https://crates.io/crates/sparse_set_container
[docs.rs badge]: https://docs.rs/sparse_set_container/badge.svg?version=1.2.1
[docs.rs link]: https://docs.rs/sparse_set_container/1.2.1/sparse_set_container/
[shields.io download count]: https://img.shields.io/crates/d/sparse_set_container.svg
<!--badge links end-->

## Usage

Add this to your Cargo.toml:
<!--install instruction start-->
```toml
[dependencies]
sparse_set_container = "1.2"
```
<!--install instruction end-->

### Description

An array-like container based on sparse set implementation that allows O(1) access to elements without hashing and allows cache-friendly iterations.

| Operation | SparseSet | Vec |
| --------- | --------- | ------- |
| push      | O(1)      | O(1)    |
| lookup    | O(1)      | O(1)    |
| len       | O(1)      | O(1)    |
| remove    | O(n)      | O(n)    |
| swap_remove | O(1)    | O(1)    |

For iterating over the elements SparseSet exposes an iterator over a tightly packed slice with values, which is as efficient as iterating over a Vec.

Differences to Vec:
- Instead of using indexes, when adding an element, it returns a lightweight key structure that can be used to access the element later
  - The key is not invalidated when elements are removed from the container
  - If the pointed-at element was removed, the key will not be pointing to any other elements, even if new elements are inserted
- There is a slight overhead in insertion/lookup/removal operations compared to Vec
- Consumes more memory:
  - for each value `4*sizeof(usize)` bytes on top of the size of the element itself
    - (e.g. 32 bytes per element on 64-bit systems)
  - per each `2^(sizeof(usize)*8)` removals the memory consumption will also grow by `2*sizeof(usize)`
    - (e.g. 16 bytes per 18446744073709551616 elements removed on 64-bit systems)
- Many Vec operations are not supported (create an [issue on github](https://github.com/gameraccoon/sparse_set_container/issues) if you want to request one)

### Examples

<!--readme_example.rs start-->
```rust
extern crate sparse_set_container;
use sparse_set_container::SparseSet;

fn main() {
    let mut elements = SparseSet::new();
    elements.push("1");
    let key2 = elements.push("2");
    elements.push("3");

    elements.remove(key2);
    elements.push("4");

    if !elements.contains(key2) {
        println!("Value 2 is not in the container");
    }

    // Prints 1 3 4
    for v in elements.values() {
        print!("{} ", v);
    }

    // Prints 1 3 4
    for k in elements.keys() {
        print!("{} ", elements.get(k).unwrap());
    }
}
```
<!--readme_example.rs end-->
### Benchmarks

The values captured illustrate the difference between this SparseSet container implementation, Vec, and standard HashMap, as well as comparing to other libraries with similar functionality.

<!--benchmark table start-->
| Benchmark                    | `SparseSet<String>` | `Vec<String>` | `HashMap<i32, String>` | `thunderdome::Arena<String>` | `generational_arena::Arena<String>` | `slotmap::SlotMap<_, String>` | `slotmap::DenseSlotMap<_, String>` |
|------------------------------|---------------------|---------------|------------------------|------------------------------|-------------------------------------|-------------------------------|------------------------------------|
| Create empty                 | 0 ns ±0             | 0 ns ±0       | 2 ns ±0                | 0 ns ±0                      | 15 ns ±0                            | 8 ns ±0                       | 8 ns ±0                            |
| Create with capacity (1000)  | 20 ns ±0            | 19 ns ±0      | 37 ns ±0               | 19 ns ±0                     | 693 ns ±2                           | 19 ns ±0                      | 53 ns ±0                           |
| Push 100 elements            | 3,612 ns ±11        | 3,499 ns ±11  | 5,537 ns ±30           | 3,631 ns ±12                 | 3,623 ns ±8                         | 3,552 ns ±13                  | 4,175 ns ±18                       |
| With capacity push 100       | 3,411 ns ±21        | 3,335 ns ±19  | 4,570 ns ±32           | 3,490 ns ±24                 | 3,418 ns ±17                        | 3,375 ns ±24                  | 3,637 ns ±14                       |
| Lookup 100 elements          | 94 ns ±1            | 44 ns ±6      | 474 ns ±25             | 83 ns ±1                     | 82 ns ±1                            | 68 ns ±1                      | 89 ns ±3                           |
| Iterate over 100 elements    | 32 ns ±0            | 32 ns ±0      | 44 ns ±0               | 98 ns ±0                     | 73 ns ±0                            | 39 ns ±2                      | 33 ns ±0                           |
| Clone with 100 elements      | 2,621 ns ±18        | 2,565 ns ±17  | 1,629 ns ±39           | 2,594 ns ±31                 | 2,659 ns ±18                        | 2,620 ns ±19                  | 2,660 ns ±14                       |
| Clone 100 and remove 10      | 3,416 ns ±75        | 2,611 ns ±43  | 1,797 ns ±113          | 2,697 ns ±77                 | 2,762 ns ±76                        | 2,730 ns ±85                  | 2,708 ns ±65                       |
| Clone 100 and swap_remove 10 | 2,698 ns ±66        | 2,401 ns ±30  | N/A                    | N/A                          | N/A                                 | N/A                           | N/A                                |
<!--benchmark table end-->

To run the benchmark on your machine, execute `cargo run --example bench --release`

Or to build this table you can run `python tools/collect_benchmark_table.py` and then find the results in `bench_table.md`

### License

Licensed under the MIT license: http://opensource.org/licenses/MIT
