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
[docs.rs badge]: https://docs.rs/sparse_set_container/badge.svg?version=1.0.1
[docs.rs link]: https://docs.rs/sparse_set_container/1.0.1/sparse_set_container/
[shields.io download count]: https://img.shields.io/crates/d/sparse_set_container.svg
<!--badge links end-->

## Usage

Add this to your Cargo.toml:
<!--install instruction start-->
```toml
[dependencies]
sparse_set_container = "1.0"
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
| Benchmark | `SparseSet<String>` | `Vec<String>` | `HashMap<i32, String>` |
| --- | --- | --- | ---|
| Create empty | 0 ns ±0 | 0 ns ±0 | 1 ns ±1 |
| Create with capacity | 54 ns ±0 | 16 ns ±0 | 33 ns ±1 |
| Push 100 elements | 3,842 ns ±95 | 2,659 ns ±101 | 5,636 ns ±191 |
| With capacity push 100 | 3,343 ns ±25 | 3,257 ns ±75 | 4,426 ns ±122 |
| Lookup 100 elements | 88 ns ±2 | 39 ns ±23 | 464 ns ±12 |
| Iterate over 100 elements | 30 ns ±0 | 30 ns ±0 | 41 ns ±1 |
| Clone with 100 elements | 2,388 ns ±22 | 2,247 ns ±12 | 1,527 ns ±60 |
| Clone 100 and remove 10 | 3,174 ns ±161 | 2,367 ns ±140 | 1,704 ns ±159 |
| Clone 100 and swap_remove 10 | 2,575 ns ±134 | 2,181 ns ±68 | N/A |
<!--benchmark table end-->

To run the benchmark on your machine, execute `cargo run --example bench --release`

Or to build this table you can run `python tools/collect_benchmark_table.py` and then find the results in `bench_table.md`

### License

Licensed under the MIT license: http://opensource.org/licenses/MIT
