use crate::internal_types;
use crate::sparse_key;

use internal_types::{AliveSparseEntry, SparseEntry};
use sparse_key::SparseKey;

/// SparseArrayStorage is a storage for sparse set, it is a combination of dense and sparse arrays.
/// Dense array stores values, sparse array stores keys to the dense array.
///
/// In the current implementation, the arrays grow at the same time, so this storage tries to
/// leverage that by allocating memory for all arrays at the same time
#[derive(Clone)]
pub(crate) struct SparseArrayStorage<T> {
    // has as many values as elements stored in the set
    dense_values: Vec<T>,
    // same size as the dense array, stores keys to the sparse array
    dense_keys: Vec<SparseKey>,
    // stores either index to the value in the dense array or index to the next free sparse slot
    sparse: Vec<SparseEntry>,
}

impl<T> SparseArrayStorage<T> {
    pub(crate) fn new() -> Self {
        Self {
            dense_values: Vec::new(),
            dense_keys: Vec::new(),
            sparse: Vec::new(),
        }
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            dense_values: Vec::with_capacity(capacity),
            dense_keys: Vec::with_capacity(capacity),
            sparse: Vec::with_capacity(capacity),
        }
    }

    // it's time to extend the storage
    pub(crate) fn add_with_new_sparse_item(&mut self, value: T) -> SparseKey {
        let key = SparseKey {
            sparse_index: self.sparse.len(),
            epoch: 0,
        };
        let new_sparse_entry = SparseEntry::AliveEntry(AliveSparseEntry {
            dense_index: self.dense_values.len(),
            epoch: 0,
        });

        self.dense_values.push(value);
        self.dense_keys.push(key);
        self.sparse.push(new_sparse_entry);

        key
    }

    pub(crate) fn add_with_existing_sparse_item(&mut self, key: SparseKey, value: T) {
        let dense_index = self.dense_values.len();

        self.dense_keys.push(key);
        self.dense_values.push(value);

        self.sparse[key.sparse_index] = SparseEntry::AliveEntry(AliveSparseEntry {
            dense_index,
            epoch: key.epoch,
        });
    }

    pub(crate) fn remove_dense(&mut self, index: usize) -> T {
        self.dense_keys.remove(index);
        self.dense_values.remove(index)
    }

    pub(crate) fn swap_remove_dense(&mut self, index: usize) -> T {
        self.dense_keys.swap_remove(index);
        self.dense_values.swap_remove(index)
    }

    pub(crate) fn get_dense_values(&self) -> &Vec<T> {
        &self.dense_values
    }

    pub(crate) fn get_dense_values_mut(&mut self) -> &mut Vec<T> {
        &mut self.dense_values
    }

    pub(crate) fn get_dense_keys(&self) -> &Vec<SparseKey> {
        &self.dense_keys
    }

    pub(crate) fn get_dense_keys_mut(&mut self) -> &mut Vec<SparseKey> {
        &mut self.dense_keys
    }

    pub(crate) fn get_sparse(&self) -> &Vec<SparseEntry> {
        &self.sparse
    }

    pub(crate) fn get_sparse_mut(&mut self) -> &mut Vec<SparseEntry> {
        &mut self.sparse
    }
}
