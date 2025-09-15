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
| Create empty                 | 0 ns ±0             | 0 ns ±0       | 1 ns ±0                | 0 ns ±0                      | 14 ns ±0                            | 7 ns ±0                       | 7 ns ±1                            |
| Create with capacity (1000)  | 19 ns ±0            | 18 ns ±0      | 35 ns ±3               | 18 ns ±0                     | 652 ns ±26                          | 18 ns ±0                      | 51 ns ±3                           |
| Push 100 elements            | 3,520 ns ±96        | 3,322 ns ±212 | 5,307 ns ±341          | 3,457 ns ±220                | 3,501 ns ±226                       | 3,352 ns ±106                 | 3,974 ns ±201                      |
| With capacity push 100       | 3,385 ns ±136       | 3,234 ns ±210 | 4,281 ns ±74           | 3,309 ns ±98                 | 3,210 ns ±70                        | 3,212 ns ±90                  | 3,377 ns ±102                      |
| Lookup 100 elements          | 89 ns ±2            | 42 ns ±7      | 447 ns ±35             | 78 ns ±2                     | 78 ns ±2                            | 64 ns ±1                      | 86 ns ±3                           |
| Iterate over 100 elements    | 30 ns ±1            | 32 ns ±2      | 42 ns ±1               | 93 ns ±2                     | 69 ns ±2                            | 36 ns ±1                      | 32 ns ±1                           |
| Clone with 100 elements      | 2,476 ns ±76        | 2,411 ns ±73  | 1,538 ns ±58           | 2,449 ns ±86                 | 2,505 ns ±81                        | 2,472 ns ±74                  | 2,496 ns ±42                       |
| Clone 100 and remove 10      | 3,215 ns ±117       | 2,454 ns ±52  | 1,678 ns ±112          | 2,539 ns ±85                 | 2,618 ns ±125                       | 2,585 ns ±86                  | 2,556 ns ±83                       |
| Clone 100 and swap_remove 10 | 2,546 ns ±74        | 2,262 ns ±86  | N/A                    | N/A                          | N/A                                 | N/A                           | N/A                                |
<!--benchmark table end-->

To run the benchmark on your machine, execute `cargo run --example bench --release`

Or to build this table you can run `python tools/collect_benchmark_table.py` and then find the results in `bench_table.md`

### License

Licensed under the MIT license: http://opensource.org/licenses/MIT
