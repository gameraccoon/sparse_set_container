# Sparse Set Container
A container based on a sparse set.

It is useful if you want a container with performance close to Vec but you also want to safely store the indexes to the elements (so that they are not invalidated on removals).  
E.g. you have a list of elements in UI that the user can add and remove, but you want to refer to the elements of that list from somewhere else.

[![crates.io][crates.io shield]][crates.io link]
[![Documentation][docs.rs badge]][docs.rs link]

[![Download Status][shields.io download count]][crates.io link]

<!--badge links start-->
[crates.io shield]: https://img.shields.io/crates/v/sparse_set_container?label=latest
[crates.io link]: https://crates.io/crates/sparse_set_container
[docs.rs badge]: https://docs.rs/sparse_set_container/badge.svg?version=1.1.1
[docs.rs link]: https://docs.rs/sparse_set_container/1.1.1/sparse_set_container/
[shields.io download count]: https://img.shields.io/crates/d/sparse_set_container.svg
<!--badge links end-->

## Usage

Add this to your Cargo.toml:
<!--install instruction start-->
```toml
[dependencies]
sparse_set_container = "1.1"
```
<!--install instruction end-->

### Description

An array-like container based on sparse set implementation that allows O(1) access to elements without hashing and allows cache-friendly iterations.

| Operation | SparseSet | Vec |
| --------- | --------- | ------- |
| push      | O(1)      | O(1)    |
| lookup    | O(1)      | O(1)    |
| size/len  | O(1)      | O(1)    |
| remove    | O(n)      | O(n)    |
| swap_remove | O(1)    | O(1)    |

For iterating over the elements SparseSet exposes an iterator over an internal Vec with values, which is as efficient as iterating over a Vec directly.

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

The values captured to illustrate the difference between this SparseSet container implementation, Vec, and standard HashMap:

<!--benchmark table start-->
| Benchmark                    | `SparseSet<String>` | `Vec<String>` | `HashMap<i32, String>` | `thunderdome::Arena<String>` | `generational_arena::Arena<String>` | `slotmap::SlotMap<_, String>` | `slotmap::DenseSlotMap<_, String>` |
|------------------------------|---------------------|---------------|------------------------|------------------------------|-------------------------------------|-------------------------------|------------------------------------|
| Create empty                 | 0 ns ±0             | 0 ns ±0       | 2 ns ±0                | 0 ns ±0                      | 14 ns ±1                            | 7 ns ±0                       | 7 ns ±0                            |
| Create with capacity (1000)  | 18 ns ±0            | 18 ns ±0      | 34 ns ±0               | 18 ns ±0                     | 653 ns ±5                           | 18 ns ±0                      | 49 ns ±0                           |
| Push 100 elements            | 3,423 ns ±23        | 3,192 ns ±13  | 5,405 ns ±33           | 3,205 ns ±15                 | 3,335 ns ±13                        | 3,198 ns ±22                  | 3,862 ns ±33                       |
| With capacity push 100       | 3,280 ns ±11        | 3,164 ns ±29  | 4,350 ns ±20           | 3,199 ns ±35                 | 3,136 ns ±10                        | 3,146 ns ±17                  | 3,309 ns ±17                       |
| Lookup 100 elements          | 88 ns ±0            | 41 ns ±6      | 464 ns ±2              | 77 ns ±1                     | 76 ns ±1                            | 64 ns ±1                      | 85 ns ±3                           |
| Iterate over 100 elements    | 30 ns ±0            | 30 ns ±0      | 41 ns ±1               | 73 ns ±0                     | 69 ns ±0                            | 36 ns ±0                      | 33 ns ±0                           |
| Clone with 100 elements      | 2,422 ns ±48        | 2,352 ns ±11  | 1,522 ns ±38           | 2,403 ns ±19                 | 2,460 ns ±17                        | 2,425 ns ±41                  | 2,442 ns ±16                       |
| Clone 100 and remove 10      | 3,183 ns ±78        | 2,408 ns ±53  | 1,673 ns ±99           | 2,516 ns ±66                 | 2,581 ns ±77                        | 2,553 ns ±72                  | 2,514 ns ±52                       |
| Clone 100 and swap_remove 10 | 2,510 ns ±61        | 2,229 ns ±35  | N/A                    | N/A                          | N/A                                 | N/A                           | N/A                                |
<!--benchmark table end-->

To run the benchmark on your machine, execute `cargo run --example bench --release`

Or to build this table you can run `python tools/collect_benchmark_table.py` and then find the results in `bench_table.md`

### License

Licensed under the MIT license: http://opensource.org/licenses/MIT
