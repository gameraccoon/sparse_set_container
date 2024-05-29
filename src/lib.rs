// Copyright (C) Pavel Grebnev 2024
// Distributed under the MIT License (license terms are at http://opensource.org/licenses/MIT).

mod internal_types;
mod sparse_key;
mod storage;

use internal_types::{AliveSparseEntry, FreeSparseEntry, SparseEntry};
pub use sparse_key::SparseKey;

/// A container based on Sparse Set, that stores a set of items and provides a way to efficiently
/// access them by a generated key.
///
/// Usage-wise it works similarly to an array, with exceptions that keys stay stable even after
/// removals, and operations like insertions and removals have slight overhead. Also, it has higher
/// memory consumption, since it needs to store additional data for each element.
///
/// Good for cache efficiency. Doesn't require any hashing. Can't be serialized.
///
/// Insertions are O(1) amortized.
/// Removals are O(1) if the order of elements can be changed, O(n) if the order must be preserved.
/// Accessing elements is O(1).
///
/// Extra memory consumption for each value is 4 * sizeof(usize) bytes on top of the size of the
/// value (e.g. 32 bytes per element on 64-bit systems).
/// The memory consumption will also grow by 2 * sizeof(usize) per 2^(sizeof(usize) * 8) elements
/// removed (e.g. 16 bytes per 18446744073709551616 elements removed on 64-bit systems).
#[derive(Clone)]
pub struct SparseSet<T> {
    // storage of dense and sparse values
    storage: storage::SparseArrayStorage<T>,
    // a "free list" of free entries in the sparse array
    next_free_sparse_entry: usize,
}

#[allow(dead_code)]
impl<T> SparseSet<T> {
    /// Does not heap-allocate when created.
    pub fn new() -> Self {
        Self {
            storage: storage::SparseArrayStorage::new(),
            next_free_sparse_entry: usize::MAX,
        }
    }

    /// Creates a new SparseSet with allocated memory for the given number of elements.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            storage: storage::SparseArrayStorage::with_capacity(capacity),
            next_free_sparse_entry: usize::MAX,
        }
    }

    /// Inserts a new value into the set and returns a key that can be used to access it.
    ///
    /// This can heap-allocate (if the internal arrays need to grow) but it won't invalidate any
    /// existing keys.
    /// If some objects were removed before, it will reclaim the previously freed space.
    ///
    /// O(1) amortized time complexity.
    pub fn push(&mut self, value: T) -> SparseKey {
        // if there are free entries in the sparse array, use one of them
        if self.next_free_sparse_entry != usize::MAX {
            let new_sparse_index = self.next_free_sparse_entry;
            let free_sparse_entry = match &self.storage.get_sparse()[new_sparse_index] {
                SparseEntry::FreeEntry(free_sparse_entry) => free_sparse_entry.clone(),
                _ => unreachable!(),
            };
            self.next_free_sparse_entry = free_sparse_entry.next_free;

            let key = SparseKey {
                sparse_index: new_sparse_index,
                epoch: free_sparse_entry.next_epoch,
            };

            self.storage.add_with_existing_sparse_item(key, value);

            key
        } else {
            // extend the sparse array
            self.storage.add_with_new_sparse_item(value)
        }
    }

    /// Removes an element from the set using the key, swapping it with the last element.
    /// Returns the removed value if it was present in the set.
    ///
    /// O(1) time complexity, however changes the order of elements.
    pub fn swap_remove(&mut self, key: SparseKey) -> Option<T> {
        // this can happen only if the key is from another SparseSet
        // in this case nothing is guaranteed anymore, we should panic
        debug_assert!(key.sparse_index < self.storage.get_sparse_mut().len());

        return match self.storage.get_sparse()[key.sparse_index].clone() {
            SparseEntry::AliveEntry(entry) if entry.epoch == key.epoch => {
                let swapped_sparse_index = self.storage.get_dense_keys()
                    [self.storage.get_dense_values().len() - 1]
                    .sparse_index;
                if let SparseEntry::AliveEntry(swapped_entry) =
                    &mut self.storage.get_sparse_mut()[swapped_sparse_index]
                {
                    swapped_entry.dense_index = entry.dense_index;
                } else {
                    unreachable!();
                }

                let removed_value = self.storage.swap_remove_dense(entry.dense_index);

                self.mark_as_free(key, entry);
                Some(removed_value)
            }
            // the element was already removed (either there's nothing, or a newer element)
            _ => None,
        };
    }

    /// Removes an element from the set using the key, keeping the order of elements.
    /// Returns the removed value if it was present in the set.
    ///
    /// O(n) time complexity, however doesn't change the order of elements.
    pub fn remove(&mut self, key: SparseKey) -> Option<T> {
        // this can happen only if the key is from another SparseSet
        // in this case nothing is guaranteed anymore, we should panic
        debug_assert!(key.sparse_index < self.storage.get_sparse().len());

        return match self.storage.get_sparse()[key.sparse_index].clone() {
            SparseEntry::AliveEntry(entry) if entry.epoch == key.epoch => {
                for i in entry.dense_index + 1..self.storage.get_dense_values().len() {
                    let sparse_index = self.storage.get_dense_keys()[i].sparse_index;
                    if let SparseEntry::AliveEntry(entry) =
                        &mut self.storage.get_sparse_mut()[sparse_index]
                    {
                        entry.dense_index -= 1;
                    } else {
                        unreachable!();
                    }
                }

                let removed_value = self.storage.remove_dense(entry.dense_index);

                self.mark_as_free(key, entry);
                Some(removed_value)
            }
            // the element was already removed (either there's nothing, or a newer element)
            _ => None,
        };
    }

    /// Swaps two elements in the set using their keys.
    ///
    /// O(1) time complexity.
    pub fn swap(&mut self, key1: SparseKey, key2: SparseKey) {
        // this can happen only if the key is from another SparseSet
        // in this case nothing is guaranteed anymore, we should panic
        debug_assert!(key1.sparse_index < self.storage.get_sparse().len());
        debug_assert!(key2.sparse_index < self.storage.get_sparse().len());

        match (
            self.storage.get_sparse()[key1.sparse_index].clone(),
            self.storage.get_sparse()[key2.sparse_index].clone(),
        ) {
            (SparseEntry::AliveEntry(entry1), SparseEntry::AliveEntry(entry2))
                if entry1.epoch == key1.epoch && entry2.epoch == key2.epoch =>
            {
                self.storage
                    .get_dense_values_mut()
                    .swap(entry1.dense_index, entry2.dense_index);
                self.storage
                    .get_dense_keys_mut()
                    .swap(entry1.dense_index, entry2.dense_index);

                // swap the references in the sparse array
                let sparse_array = self.storage.get_sparse_mut();
                sparse_array[key1.sparse_index] = SparseEntry::AliveEntry(AliveSparseEntry {
                    dense_index: entry2.dense_index,
                    epoch: entry1.epoch,
                });
                sparse_array[key2.sparse_index] = SparseEntry::AliveEntry(AliveSparseEntry {
                    dense_index: entry1.dense_index,
                    epoch: entry2.epoch,
                });
            }
            // either there's no element, or there's a newer element the value points to
            _ => {
                panic!("Cannot swap elements that are not alive");
            }
        }
    }

    /// Returns a reference to the value stored at the given key.
    /// If the key is not valid, returns None.
    ///
    /// O(1) time complexity.
    pub fn get(&self, key: SparseKey) -> Option<&T> {
        // this can happen only if the key is from another SparseSet
        // in this case nothing is guaranteed anymore, we should panic
        debug_assert!(key.sparse_index < self.storage.get_sparse().len());

        match &self.storage.get_sparse()[key.sparse_index] {
            SparseEntry::AliveEntry(entry) if entry.epoch == key.epoch => {
                Some(&self.storage.get_dense_values()[entry.dense_index])
            }
            // either there's no element, or there's a newer element the value points to
            _ => None,
        }
    }

    /// Returns a mutable reference to the value stored at the given key.
    /// If the key is not valid, returns None.
    ///
    /// O(1) time complexity.
    pub fn get_mut(&mut self, key: SparseKey) -> Option<&mut T> {
        // this can happen only if the key is from another SparseSet
        // in this case nothing is guaranteed anymore, we should panic
        debug_assert!(key.sparse_index < self.storage.get_sparse().len());

        match self.storage.get_sparse()[key.sparse_index].clone() {
            SparseEntry::AliveEntry(entry) if entry.epoch == key.epoch => {
                Some(&mut self.storage.get_dense_values_mut()[entry.dense_index])
            }
            // either there's no element, or there's a newer element the value points to
            _ => None,
        }
    }

    /// Returns true if the key points to a valid element in the set.
    ///
    /// O(1) time complexity.
    pub fn contains(&self, key: SparseKey) -> bool {
        if key.sparse_index >= self.storage.get_sparse().len() {
            debug_assert!(false, "The key is not valid for this SparseSet");
            return false;
        }

        match &self.storage.get_sparse()[key.sparse_index] {
            SparseEntry::AliveEntry(entry) if entry.epoch == key.epoch => true,
            _ => false,
        }
    }

    /// Returns the number of elements in the set.
    ///
    /// O(1) time complexity.
    pub fn size(&self) -> usize {
        self.storage.get_dense_values().len()
    }

    /// Returns true if the set is empty.
    ///
    /// O(1) time complexity.
    pub fn is_empty(&self) -> bool {
        self.storage.get_dense_values().is_empty()
    }

    /// Returns an iterator over the values of the set.
    pub fn values(&self) -> impl DoubleEndedIterator<Item = &T> {
        self.storage.get_dense_values().iter()
    }

    /// Returns an iterator over the mutable values of the set.
    pub fn values_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut T> {
        self.storage.get_dense_values_mut().iter_mut()
    }

    /// Returns an iterator over the keys of the set.
    pub fn keys(&self) -> impl DoubleEndedIterator<Item = SparseKey> + '_ {
        self.storage.get_dense_keys().iter().copied()
    }

    /// Returns an iterator over the key-value pairs of the set.
    ///
    /// Note that if you intend to iterate over key-values in time-critical code, it may be better
    /// to instead store the keys in the elements themselves to reduce CPU cache misses.
    pub fn key_values(&self) -> impl DoubleEndedIterator<Item = (SparseKey, &T)> {
        self.storage
            .get_dense_keys()
            .iter()
            .copied()
            .zip(self.storage.get_dense_values().iter())
    }

    fn mark_as_free(&mut self, key: SparseKey, entry: AliveSparseEntry) {
        self.storage.get_sparse_mut()[key.sparse_index] = SparseEntry::FreeEntry(FreeSparseEntry {
            next_free: self.next_free_sparse_entry,
            next_epoch: usize::wrapping_add(entry.epoch, 1),
        });

        // as long as we have available epochs, we can reuse the sparse entry
        if key.epoch < usize::MAX {
            self.next_free_sparse_entry = key.sparse_index;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // empty sparse set => created => no items
    #[test]
    fn empty_sparse_set_created_no_items() {
        let sparse_set: SparseSet<i32> = SparseSet::new();

        assert_eq!(sparse_set.size(), 0);
        for _ in sparse_set.values() {
            assert!(false);
        }
    }

    // empty sparse set => created with capacity => no items
    #[test]
    fn empty_sparse_set_created_with_capacity_no_items() {
        let sparse_set: SparseSet<i32> = SparseSet::with_capacity(10);

        assert_eq!(sparse_set.size(), 0);
        for _ in sparse_set.values() {
            assert!(false);
        }
    }

    // empty sparse set => push item => has one item
    #[test]
    fn empty_sparse_set_push_item_has_one_item() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();

        let key = sparse_set.push(42);

        assert_eq!(sparse_set.size(), 1);
        assert_eq!(sparse_set.get(key), Some(&42));
    }

    // empty sparse set with capacity => push item => has one item
    #[test]
    fn empty_sparse_set_with_capacity_push_item_has_one_item() {
        let mut sparse_set: SparseSet<i32> = SparseSet::with_capacity(10);

        let key = sparse_set.push(42);

        assert_eq!(sparse_set.size(), 1);
        assert_eq!(sparse_set.get(key), Some(&42));
    }

    // sparse set with one item => mutate the item => the item is changed
    #[test]
    fn sparse_set_with_one_item_mutate_the_item_the_item_is_changed() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();
        let key = sparse_set.push(42);

        *sparse_set.get_mut(key).unwrap() = 43;

        assert_eq!(sparse_set.size(), 1);
        assert_eq!(sparse_set.get(key), Some(&43));
    }

    // sparse set with one item => remove item => no items
    #[test]
    fn sparse_set_with_one_item_remove_item_no_items() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();
        let key = sparse_set.push(42);

        sparse_set.remove(key);

        assert_eq!(sparse_set.size(), 0);
        assert_eq!(sparse_set.get(key), None);
    }

    // sparse set with one item => swap_remove item => no items
    #[test]
    fn swap_sparse_set_with_one_item_remove_item_no_items() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();
        let key = sparse_set.push(42);

        sparse_set.swap_remove(key);

        assert_eq!(sparse_set.size(), 0);
        assert_eq!(sparse_set.get(key), None);
    }

    // sparse set with two items => remove first item => has one item
    #[test]
    fn sparse_set_with_two_items_remove_first_item_has_one_item() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();
        let key1 = sparse_set.push(42);
        let key2 = sparse_set.push(43);

        sparse_set.remove(key1);

        assert_eq!(sparse_set.size(), 1);
        assert_eq!(sparse_set.get(key1), None);
        assert_eq!(sparse_set.get(key2), Some(&43));
    }

    // sparse set with two items => swap_remove first item => has one item
    #[test]
    fn swap_sparse_set_with_two_items_remove_first_item_has_one_item() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();
        let key1 = sparse_set.push(42);
        let key2 = sparse_set.push(43);

        sparse_set.swap_remove(key1);

        assert_eq!(sparse_set.size(), 1);
        assert_eq!(sparse_set.get(key1), None);
        assert_eq!(sparse_set.get(key2), Some(&43));
    }

    // sparse set with two items => remove second item => has one item
    #[test]
    fn sparse_set_with_two_items_remove_second_item_has_one_item() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();
        let key1 = sparse_set.push(42);
        let key2 = sparse_set.push(43);

        sparse_set.remove(key2);

        assert_eq!(sparse_set.size(), 1);
        assert_eq!(sparse_set.get(key1), Some(&42));
        assert_eq!(sparse_set.get(key2), None);
    }

    // sparse set with two items => swap_remove second item => has one item
    #[test]
    fn swap_sparse_set_with_two_items_remove_second_item_has_one_item() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();
        let key1 = sparse_set.push(42);
        let key2 = sparse_set.push(43);

        sparse_set.swap_remove(key2);

        assert_eq!(sparse_set.size(), 1);
        assert_eq!(sparse_set.get(key1), Some(&42));
        assert_eq!(sparse_set.get(key2), None);
    }

    // spare set with one item => remove an item and push new item => has one item
    #[test]
    fn sparse_set_with_one_item_remove_an_item_and_push_new_item_has_one_item() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();
        let key = sparse_set.push(42);
        sparse_set.remove(key);

        let new_key = sparse_set.push(43);

        assert_eq!(sparse_set.size(), 1);
        assert_eq!(sparse_set.get(key), None);
        assert_eq!(sparse_set.get(new_key), Some(&43));
    }

    // sparse set with one item => swap_remove an item and push new item => has one item
    #[test]
    fn swap_sparse_set_with_one_item_remove_an_item_and_push_new_item_has_one_item() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();
        let key = sparse_set.push(42);
        sparse_set.swap_remove(key);

        let new_key = sparse_set.push(43);

        assert_eq!(sparse_set.size(), 1);
        assert_eq!(sparse_set.get(key), None);
        assert_eq!(sparse_set.get(new_key), Some(&43));
    }

    // sparse set with five items => remove first item => order is not changed
    #[test]
    fn sparse_set_with_five_items_remove_first_item_order_is_not_changed() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();
        let key1 = sparse_set.push(42);
        let key2 = sparse_set.push(43);
        let key3 = sparse_set.push(44);
        let key4 = sparse_set.push(45);
        let key5 = sparse_set.push(46);

        sparse_set.remove(key1);

        assert_eq!(sparse_set.size(), 4);
        for (i, value) in sparse_set.values().enumerate() {
            if i == 0 {
                assert_eq!(value, &43);
            } else if i == 1 {
                assert_eq!(value, &44);
            } else if i == 2 {
                assert_eq!(value, &45);
            } else {
                assert_eq!(value, &46);
            }
        }
        assert_eq!(sparse_set.get(key1), None);
        assert_eq!(sparse_set.get(key2), Some(&43));
        assert_eq!(sparse_set.get(key3), Some(&44));
        assert_eq!(sparse_set.get(key4), Some(&45));
        assert_eq!(sparse_set.get(key5), Some(&46));
    }

    // sparse set with one item => remove item twice => no items
    #[test]
    fn sparse_set_with_one_item_remove_item_twice_no_items() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();
        let key = sparse_set.push(42);

        sparse_set.remove(key);
        sparse_set.remove(key);

        assert_eq!(sparse_set.size(), 0);
        assert_eq!(sparse_set.get(key), None);
    }

    // sparse set with one item => remove item twice => no items
    #[test]
    fn sparse_set_with_one_item_swap_remove_item_twice_no_items() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();
        let key = sparse_set.push(42);

        sparse_set.swap_remove(key);
        sparse_set.swap_remove(key);

        assert_eq!(sparse_set.size(), 0);
        assert_eq!(sparse_set.get(key), None);
    }

    // sparse set with three items => iterate over values => the values are iterated in order
    #[test]
    fn sparse_set_with_three_items_iterate_over_values_the_values_are_iterated_in_order() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();
        sparse_set.push(42);
        sparse_set.push(43);
        sparse_set.push(44);

        for (i, value) in sparse_set.values().enumerate() {
            if i == 0 {
                assert_eq!(value, &42);
            } else if i == 1 {
                assert_eq!(value, &43);
            } else {
                assert_eq!(value, &44);
            }
        }
    }

    // sparse set with three items => iterate over keys => the keys are iterated in order
    #[test]
    fn sparse_set_with_three_items_iterate_over_keys_the_keys_are_iterated_in_order() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();
        sparse_set.push(42);
        sparse_set.push(43);
        sparse_set.push(44);

        for (i, key) in sparse_set.keys().enumerate() {
            if i == 0 {
                assert_eq!(sparse_set.get(key), Some(&42));
            } else if i == 1 {
                assert_eq!(sparse_set.get(key), Some(&43));
            } else {
                assert_eq!(sparse_set.get(key), Some(&44));
            }
        }
    }

    // sparse set with three items => iterate over key-values => the key-values are iterated in order
    #[test]
    fn sparse_set_with_three_items_iterate_over_key_values_the_key_values_are_iterated_in_order() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();
        let key1 = sparse_set.push(42);
        let key2 = sparse_set.push(43);
        let key3 = sparse_set.push(44);

        for (i, (key, value)) in sparse_set.key_values().enumerate() {
            if i == 0 {
                assert_eq!(value, &42);
                assert_eq!(key, key1);
            } else if i == 1 {
                assert_eq!(value, &43);
                assert_eq!(key, key2);
            } else {
                assert_eq!(value, &44);
                assert_eq!(key, key3);
            }
        }
    }

    // sparse set with one item => iterate over values and mutate => the value is changed
    #[test]
    fn sparse_set_with_one_item_iterate_over_values_and_mutate_the_value_is_changed() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();
        let key = sparse_set.push(42);

        for value in sparse_set.values_mut() {
            *value = 43;
        }

        assert_eq!(sparse_set.size(), 1);
        assert_eq!(sparse_set.get(key), Some(&43));
    }

    // sparse set with two items => swap the items => the items are swapped in order but not by keys
    #[test]
    fn sparse_set_with_two_items_swap_the_items_the_items_are_swapped_in_order_but_not_by_keys() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();
        let key1 = sparse_set.push(42);
        let key2 = sparse_set.push(43);

        sparse_set.swap(key1, key2);

        assert_eq!(sparse_set.size(), 2);
        for (i, value) in sparse_set.values().enumerate() {
            if i == 0 {
                assert_eq!(value, &43);
            } else {
                assert_eq!(value, &42);
            }
        }
        assert_eq!(sparse_set.get(key1), Some(&42));
        assert_eq!(sparse_set.get(key2), Some(&43));
    }

    // sparse set with two items => clone the set => cloned set has the same items
    #[test]
    fn sparse_set_with_two_items_clone_the_set_cloned_set_has_the_same_items() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();
        let key1 = sparse_set.push(42);
        let key2 = sparse_set.push(43);

        let cloned_sparse_set = sparse_set.clone();

        assert_eq!(cloned_sparse_set.size(), 2);
        assert_eq!(cloned_sparse_set.get(key1), Some(&42));
        assert_eq!(cloned_sparse_set.get(key2), Some(&43));
    }

    // sparse set with one item => check if contains => returns true
    #[test]
    fn sparse_set_with_one_item_check_if_contains_returns_true() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();
        let key = sparse_set.push(42);

        assert!(sparse_set.contains(key));
    }

    // sparse set with one item => remove the item and check if contains => returns false
    #[test]
    fn sparse_set_with_one_item_remove_the_item_and_check_if_contains_returns_false() {
        let mut sparse_set: SparseSet<i32> = SparseSet::new();
        let key = sparse_set.push(42);

        sparse_set.swap_remove(key);

        assert!(!sparse_set.contains(key));
    }
}
