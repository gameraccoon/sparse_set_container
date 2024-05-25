#[macro_use]
extern crate bencher;

use bencher::{black_box, Bencher};
use sparse_set_container::SparseSet;
use std::collections::HashMap;

// 100 random indexes
static INDEXES: [usize; 100] = [
    69, 47, 15, 48, 51, 32, 75, 88, 28, 61, 75, 92, 75, 26, 79, 7, 19, 62, 5, 55, 23, 94, 37, 83,
    78, 99, 38, 87, 60, 77, 81, 19, 96, 61, 78, 47, 39, 74, 3, 65, 12, 29, 78, 61, 92, 71, 70, 71,
    38, 27, 97, 46, 20, 3, 47, 75, 6, 97, 37, 27, 23, 88, 44, 30, 87, 31, 17, 54, 26, 34, 15, 3,
    24, 42, 21, 15, 35, 65, 72, 37, 9, 45, 94, 45, 71, 3, 64, 67, 27, 36, 82, 9, 78, 86, 94, 35,
    62, 47, 99, 34,
];

static REMOVABLE_INDEXES: [usize; 10] = [69, 47, 15, 48, 51, 32, 75, 38, 28, 61];

fn create_empty_sparse_set(b: &mut Bencher) {
    b.iter(|| {
        let set = SparseSet::<String>::new();
        black_box(&set);
    });
}

fn create_empty_vec(b: &mut Bencher) {
    b.iter(|| {
        let vec = Vec::<String>::new();
        black_box(&vec);
    });
}

fn create_empty_hash_map(b: &mut Bencher) {
    b.iter(|| {
        let map = HashMap::<i32, String>::new();
        black_box(&map);
    });
}

fn create_with_capacity_sparse_set(b: &mut Bencher) {
    b.iter(|| {
        let set = SparseSet::<String>::with_capacity(1000);
        black_box(&set);
    });
}

fn create_with_capacity_vec(b: &mut Bencher) {
    b.iter(|| {
        let vec = Vec::<String>::with_capacity(1000);
        black_box(&vec);
    });
}

fn create_with_capacity_hash_map(b: &mut Bencher) {
    b.iter(|| {
        let map = HashMap::<i32, String>::with_capacity(1000);
        black_box(&map);
    });
}

fn push_hundred_elements_sparse_set(b: &mut Bencher) {
    b.iter(|| {
        let mut set = SparseSet::<String>::new();
        for i in INDEXES.iter() {
            set.push(i.to_string());
        }
        black_box(&set);
    });
}

fn push_hundred_elements_vec(b: &mut Bencher) {
    b.iter(|| {
        let mut vec = Vec::<String>::new();
        for i in INDEXES.iter() {
            vec.push(i.to_string());
        }
        black_box(&vec);
    });
}

fn push_hundred_elements_hash_map(b: &mut Bencher) {
    b.iter(|| {
        let mut map = HashMap::<i32, String>::new();
        for i in INDEXES.iter() {
            map.insert(*i as i32, i.to_string());
        }
        black_box(&map);
    });
}

fn get_hundred_elements_sparse_set(b: &mut Bencher) {
    let mut set = SparseSet::<String>::new();
    let mut keys = Vec::new();
    for i in INDEXES.iter() {
        keys.push(set.push(i.to_string()));
    }
    black_box(&mut keys);
    black_box(&mut set);
    b.iter(|| {
        for i in INDEXES.iter() {
            black_box(&set.get(keys[*i]));
        }
    });
}

fn get_hundred_elements_vec(b: &mut Bencher) {
    let mut vec = Vec::<String>::new();
    for i in INDEXES.iter() {
        vec.push(i.to_string());
    }
    black_box(&mut vec);
    b.iter(|| {
        for i in INDEXES.iter() {
            black_box(&vec[*i]);
        }
    });
}

fn get_hundred_elements_hash_map(b: &mut Bencher) {
    let mut map = HashMap::<i32, String>::new();
    for i in INDEXES.iter() {
        map.insert(*i as i32, i.to_string());
    }
    black_box(&mut map);
    b.iter(|| {
        for i in INDEXES.iter() {
            black_box(&map.get(&(*i as i32)));
        }
    });
}

fn iterate_over_hundred_elements_sparse_set(b: &mut Bencher) {
    let mut set = SparseSet::<String>::new();
    for i in INDEXES.iter() {
        set.push(i.to_string());
    }
    black_box(&mut set);
    b.iter(|| {
        for element in set.values() {
            black_box(element);
        }
    });
}

fn iterate_over_hundred_elements_vec(b: &mut Bencher) {
    let mut vec = Vec::<String>::new();
    for i in INDEXES.iter() {
        vec.push(i.to_string());
    }
    black_box(&mut vec);
    b.iter(|| {
        for element in vec.iter() {
            black_box(element);
        }
    });
}

fn iterate_over_hundred_elements_hash_map(b: &mut Bencher) {
    let mut map = HashMap::<i32, String>::new();
    for i in INDEXES.iter() {
        map.insert(*i as i32, i.to_string());
    }
    black_box(&mut map);
    b.iter(|| {
        for element in map.values() {
            black_box(element);
        }
    });
}

fn clone_with_hundred_elements_sparse_set(b: &mut Bencher) {
    let mut set = SparseSet::<String>::new();
    for i in INDEXES.iter() {
        set.push(i.to_string());
    }
    black_box(&mut set);
    b.iter(|| {
        let cloned = set.clone();
        black_box(&cloned);
    });
}

fn clone_with_hundred_elements_vec(b: &mut Bencher) {
    let mut vec = Vec::<String>::new();
    for i in INDEXES.iter() {
        vec.push(i.to_string());
    }
    black_box(&mut vec);
    b.iter(|| {
        let cloned = vec.clone();
        black_box(&cloned);
    });
}

fn clone_with_hundred_elements_hash_map(b: &mut Bencher) {
    let mut map = HashMap::<i32, String>::new();
    for i in INDEXES.iter() {
        map.insert(*i as i32, i.to_string());
    }
    black_box(&mut map);
    b.iter(|| {
        let cloned = map.clone();
        black_box(&cloned);
    });
}

fn clone_and_remove_ten_out_of_hundred_elements_sparse_set(b: &mut Bencher) {
    let mut set = SparseSet::<String>::new();
    let mut keys = Vec::new();
    for i in INDEXES.iter() {
        keys.push(set.push(i.to_string()));
    }
    black_box(&mut keys);
    b.iter(|| {
        let mut cloned = set.clone();
        black_box(&mut cloned);
        for i in REMOVABLE_INDEXES.iter() {
            cloned.remove(keys[*i]);
        }
        black_box(&cloned);
    });
}

fn clone_and_remove_ten_out_of_hundred_elements_vec(b: &mut Bencher) {
    let mut vec = Vec::<String>::new();
    for i in INDEXES.iter() {
        vec.push(i.to_string());
    }
    b.iter(|| {
        let mut cloned = vec.clone();
        black_box(&mut cloned);
        for i in REMOVABLE_INDEXES.iter() {
            cloned.remove(*i);
        }
        black_box(&cloned);
    });
}

fn clone_and_remove_ten_out_of_hundred_elements_hash_map(b: &mut Bencher) {
    let mut map = HashMap::<i32, String>::new();
    for i in INDEXES.iter() {
        map.insert(*i as i32, i.to_string());
    }
    b.iter(|| {
        let mut cloned = map.clone();
        black_box(&mut cloned);
        for i in REMOVABLE_INDEXES.iter() {
            cloned.remove(&(*i as i32));
        }
        black_box(&cloned);
    });
}

fn clone_and_swap_remove_ten_out_of_hundred_elements_sparse_set(b: &mut Bencher) {
    let mut set = SparseSet::<String>::new();
    let mut keys = Vec::new();
    for i in INDEXES.iter() {
        keys.push(set.push(i.to_string()));
    }
    black_box(&mut keys);
    b.iter(|| {
        let mut cloned = set.clone();
        black_box(&mut cloned);
        for i in REMOVABLE_INDEXES.iter() {
            cloned.swap_remove(keys[*i]);
        }
        black_box(&cloned);
    });
}

fn clone_and_swap_remove_ten_out_of_hundred_elements_vec(b: &mut Bencher) {
    let mut vec = Vec::<String>::new();
    for i in INDEXES.iter() {
        vec.push(i.to_string());
    }
    b.iter(|| {
        let mut cloned = vec.clone();
        black_box(&mut cloned);
        for i in REMOVABLE_INDEXES.iter() {
            cloned.swap_remove(*i);
        }
        black_box(&cloned);
    });
}

benchmark_group!(
    benches,
    create_empty_sparse_set,
    create_empty_vec,
    create_empty_hash_map,
    create_with_capacity_sparse_set,
    create_with_capacity_vec,
    create_with_capacity_hash_map,
    push_hundred_elements_sparse_set,
    push_hundred_elements_vec,
    push_hundred_elements_hash_map,
    get_hundred_elements_sparse_set,
    get_hundred_elements_vec,
    get_hundred_elements_hash_map,
    iterate_over_hundred_elements_sparse_set,
    iterate_over_hundred_elements_vec,
    iterate_over_hundred_elements_hash_map,
    clone_with_hundred_elements_sparse_set,
    clone_with_hundred_elements_vec,
    clone_with_hundred_elements_hash_map,
    clone_and_remove_ten_out_of_hundred_elements_sparse_set,
    clone_and_remove_ten_out_of_hundred_elements_vec,
    clone_and_remove_ten_out_of_hundred_elements_hash_map,
    clone_and_swap_remove_ten_out_of_hundred_elements_sparse_set,
    clone_and_swap_remove_ten_out_of_hundred_elements_vec,
);
benchmark_main!(benches);
