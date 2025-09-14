// Copyright (C) Pavel Grebnev 2024
// Distributed under the MIT License (license terms are at http://opensource.org/licenses/MIT).

/// A sparse entry in the sparse set.
/// Depending on the state of the entry, the fields have different meanings:
/// - If the entry is alive:
///   - `dense_index_or_next_free` is the dense index of the entry.
///   - `epoch_or_next_epoch` is the epoch of the entry.
/// - If the entry is free:
///   - `dense_index_or_next_free` is the next free entry.
///   - `epoch_or_next_epoch` is the next epoch.
///
/// The index in the free entry has the upper bit set to 1, that upper bit is used to
/// differentiate between alive and free entries.
#[derive(Copy, Clone)]
pub(crate) struct SparseEntry {
    /// alive: dense_index, free: next_free
    dense_index_or_next_free: usize,
    /// alive: epoch, free: next_epoch
    epoch_or_next_epoch: usize,
}

const DEAD_BIT: usize = 1 << (size_of::<usize>() * 8 - 1);

// The max possible value next_free can have due to the dead bit being always unset
pub(crate) const MAX_SPARSE_INDEX: usize = DEAD_BIT - 1;

pub(crate) const MAX_EPOCH: usize = usize::MAX;

impl SparseEntry {
    pub(crate) fn new_alive(dense_index: usize, epoch: usize) -> Self {
        Self {
            dense_index_or_next_free: dense_index,
            epoch_or_next_epoch: epoch,
        }
    }

    pub(crate) fn mark_free(&mut self, next_free: usize) {
        debug_assert!(self.is_alive());
        self.dense_index_or_next_free = next_free | DEAD_BIT;
        self.epoch_or_next_epoch = usize::wrapping_add(self.epoch_or_next_epoch, 1);
    }

    pub(crate) fn replace_pointed_to_value(&mut self, new_dense_index: usize) {
        debug_assert!(self.is_alive());
        self.dense_index_or_next_free = new_dense_index;
    }

    pub(crate) fn is_alive(&self) -> bool {
        // use the dead bit to differentiate between alive and free entries
        self.dense_index_or_next_free & DEAD_BIT == 0
    }

    pub(crate) fn dense_index(&self) -> usize {
        debug_assert!(self.is_alive());
        self.dense_index_or_next_free
    }

    pub(crate) fn epoch(&self) -> usize {
        debug_assert!(self.is_alive());
        self.epoch_or_next_epoch
    }

    pub(crate) fn next_free(&self) -> usize {
        debug_assert!(!self.is_alive());
        self.dense_index_or_next_free & !DEAD_BIT
    }

    pub(crate) fn next_epoch(&self) -> usize {
        debug_assert!(!self.is_alive());
        self.epoch_or_next_epoch
    }

    pub(crate) fn set_dense_index(&mut self, dense_index: usize) {
        debug_assert!(self.is_alive());
        self.dense_index_or_next_free = dense_index;
    }

    pub(crate) fn dense_index_move_left(&mut self) {
        debug_assert!(self.is_alive());
        self.dense_index_or_next_free -= 1;
    }
}
