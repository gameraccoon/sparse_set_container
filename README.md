# Sparse Set Container
A container based on a sparse set

[![crates.io][crates.io shield]][crates.io link]
[![Documentation][docs.rs badge]][docs.rs link]

[![Download Status][shields.io download count]][crates.io link]

[crates.io shield]: https://img.shields.io/crates/v/sparse_set_container?label=latest
[crates.io link]: https://crates.io/crates/sparse_set_container
[docs.rs badge]: https://docs.rs/sparse_set_container/badge.svg?version=0.4.0
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

For iterating over the elements SparseSetContainer exposes an iterator over an internal Vec with values, which is as efficient as iterating over a Vec directly.

Differences to Vec:
- Instead of using indexes, when adding an element, it returns a lightweight key structure that can be used to access the element later
  - The key is not invalidated when elements are removed from the container
  - If the pointed-at element was removed, the key will not be pointing to any other elements, even if new elements are inserted
- There is a slight overhead in insertion/lookup/removal operations compared to Vec, though the overhead is constant
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

### License

Licensed under the MIT license: http://opensource.org/licenses/MIT
