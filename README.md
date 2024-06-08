# Sparse Set Container
A container based on a sparse set

[![crates.io][crates.io shield]][crates.io link]
[![Documentation][docs.rs badge]][docs.rs link]

[![Download Status][shields.io download count]][crates.io link]

[crates.io shield]: https://img.shields.io/crates/v/sparse_set_container?label=latest
[crates.io link]: https://crates.io/crates/sparse_set_container
[docs.rs badge]: https://docs.rs/sparse_set_container/badge.svg?version=1.0.0
[docs.rs link]: https://docs.rs/sparse_set_container/1.0.0/sparse_set_container/
[shields.io download count]: https://img.shields.io/crates/d/sparse_set_container.svg

## Usage

Add this to your Cargo.toml:

```toml
[dependencies]
sparse_set_container = "1.0"
```

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

### When it is useful

If you want to have a Vec of your elements but also want to store indexes to it in a safe way.  
E.g. you have a list of elements in UI that the user can add and remove, but you want to refer to the elements from that list from somewhere else.

### Examples

```rust
use sparse_set_container::SparseSet;

let mut elements = SparseSet::new();
elements.push("1");
let key2 = elements.push("2");
elements.push("3");

elements.remove(key2);
elements.push("4")

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
```
### Benchmarks

The values captured to illustrate the difference between this SparseSet container implementation, Vec, and standard HashMap:

| Test                         | `SparseSet<String>` | `Vec<String>`     | `HashMap<i32, String>` |
|------------------------------|---------------------|-------------------|------------------------|
| Create empty                 | 0 ns ±0             | 0 ns ±0           | 2 ns ±0                |
| Create with capacity         | 54 ns ±1            | 16 ns ±1          | 33 ns ±1               |
| Push 100 elements            | 3,860 ns ±112       | 3,159 ns ±97      | 5,528 ns ±249          |
| With capacity push 100       | 3,307 ns ±88        | 3,234 ns ±93      | 4,430 ns ±95           |
| Lookup 100 elements          | 88 ns ±1            | 35 ns ±22         | 464 ns ±14             |
| Iterate over 100 elements    | 30 ns ±0            | 30 ns ±0          | 41 ns ±1               |
| Clone with 100 elements      | 2,364 ns ±52        | 2,261 ns ±59      | 1,511 ns ±52           |
| Clone 100 and remove 10      | 3,215 ns ±141       | 2,697 ns ±128     | 1,683 ns ±182          |
| Clone 100 and swap_remove 10 | 2,595 ns ±106       | 2,462 ns ±126     | N/A                    |
| (~) remove 10 from 100       | 851 ms ±193         | 436 ms ±187       | 172 ms ±234            |
| (~) swap_remove 10 from 100  | 231 ms ±158         | 201 ms ±185       | N/A                    |

(~) calculated by subtracting time to clone from clone+remove, can be highly inaccurate.

To run the benchmark on your machine, run `cargo run --example bench --release`

### License

Licensed under the MIT license: http://opensource.org/licenses/MIT
