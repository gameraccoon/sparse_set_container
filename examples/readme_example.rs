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
