// Copyright (C) Pavel Grebnev 2024-2025
// Distributed under the MIT License (license terms are at http://opensource.org/licenses/MIT).

use crate::sparse_entry;
use crate::sparse_key;

use sparse_entry::SparseEntry;
use sparse_key::SparseKey;

/// SparseArrayStorage is a storage for sparse set, it is a combination of dense and sparse arrays.
/// Dense array stores values, sparse array stores keys to the dense array.
pub(crate) struct SparseArrayStorage<T> {
    // pointer to the start of the dense values array
    dense_values_start_ptr: *mut T,
    // pointer to the dense keys array
    dense_keys_start_ptr: *mut SparseKey,
    // amount of used elements in the dense array
    dense_len: usize,
    // pointer to the sparse array
    sparse_start_ptr: *mut SparseEntry,
    // amount of used elements in the sparse array
    sparse_len: usize,

    // number of elements that can fit into the allocated buffer
    max_dense_elements: usize,
    max_sparse_elements: usize,

    // pointer to the start of the allocated buffer
    buffer: *mut u8,
    // last allocation layout
    layout: Option<std::alloc::Layout>,
}

impl<T> SparseArrayStorage<T> {
    // don't waste space for big objects, and for smaller ones don't waste time on early reallocations
    const MIN_NON_ZERO_CAPACITY: usize = if size_of::<T>() <= 1024 { 4 } else { 1 };

    pub(crate) fn new() -> Self {
        Self {
            dense_values_start_ptr: std::ptr::NonNull::dangling().as_ptr(),
            dense_keys_start_ptr: std::ptr::NonNull::dangling().as_ptr(),
            dense_len: 0,
            sparse_start_ptr: std::ptr::NonNull::dangling().as_ptr(),
            sparse_len: 0,

            max_dense_elements: 0,
            max_sparse_elements: 0,

            buffer: std::ptr::null_mut(),
            layout: None,
        }
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            return Self::new();
        }

        let (layout, buffer, dense_keys_offset, sparse_offset) =
            Self::allocate_new_buffer(size_of::<T>(), align_of::<T>(), capacity, capacity);

        Self {
            dense_values_start_ptr: buffer as *mut T,
            dense_keys_start_ptr: unsafe { buffer.add(dense_keys_offset) as *mut SparseKey },
            dense_len: 0,
            sparse_start_ptr: unsafe { buffer.add(sparse_offset) as *mut SparseEntry },
            sparse_len: 0,

            max_dense_elements: capacity,
            max_sparse_elements: capacity,

            buffer,
            layout,
        }
    }

    /// # Safety
    ///
    /// - Providing position out of bounds of alive keys/values can lead to UB
    /// - Calling this function when there are free sparse entries left
    /// can lead to an inconsistent state of the storage that can later lead to UB
    pub(crate) fn insert_with_new_sparse_item(&mut self, position: usize, value: T) -> SparseKey {
        let old_sparse_len = self.sparse_len;

        let key = SparseKey {
            sparse_index: old_sparse_len,
            epoch: 0,
        };
        let new_sparse_entry = SparseEntry::new_alive(position, 0);

        if self.sparse_len == self.max_sparse_elements {
            if self.max_sparse_elements != 0 {
                self.reserve(self.max_sparse_elements);
            } else {
                self.reserve(Self::MIN_NON_ZERO_CAPACITY);
            }
        }

        if position != self.dense_len {
            self.shift_dense_values_to_the_right(position, self.dense_len, 1);
        }

        unsafe {
            std::ptr::write(self.dense_values_start_ptr.add(position), value);
            std::ptr::write(self.dense_keys_start_ptr.add(position), key);
            std::ptr::write(self.sparse_start_ptr.add(old_sparse_len), new_sparse_entry);
        }

        self.dense_len += 1;
        self.sparse_len += 1;

        key
    }

    /// # Safety
    ///
    /// Providing position out of bounds of alive keys/values can lead to UB
    pub(crate) fn insert_with_existing_sparse_item(
        &mut self,
        position: usize,
        key: SparseKey,
        value: T,
    ) {
        // no need to grow the buffer, since we know that if we have sparse entities available
        // we also have space in the dense array

        if position != self.dense_len {
            self.shift_dense_values_to_the_right(position, self.dense_len, 1);
        }

        unsafe {
            std::ptr::write(self.dense_values_start_ptr.add(position), value);
            std::ptr::write(self.dense_keys_start_ptr.add(position), key);
        }

        self.dense_len += 1;

        unsafe {
            let sparse_entry = self.sparse_start_ptr.add(key.sparse_index);
            *sparse_entry = SparseEntry::new_alive(position, key.epoch);
        }
    }

    /// # Safety
    ///
    /// Providing incorrect index will lead to UB
    pub(crate) fn remove_dense(&mut self, index: usize) -> T {
        let old_dense_len = self.dense_len;

        // move the element from the storage to the local variable
        let value = unsafe { std::ptr::read(self.dense_values_start_ptr.add(index)) };
        // key doesn't need to be explicitly dropped
        let dense_values_span_start = unsafe { self.dense_values_start_ptr.add(index) };
        let dense_keys_span_start = unsafe { self.dense_keys_start_ptr.add(index) };

        let elements_to_shift = old_dense_len - index - 1;

        unsafe {
            std::ptr::copy(
                dense_values_span_start.add(1),
                dense_values_span_start,
                elements_to_shift,
            );
            std::ptr::copy(
                dense_keys_span_start.add(1),
                dense_keys_span_start,
                elements_to_shift,
            );
        }

        self.dense_len -= 1;

        value
    }

    /// # Safety
    ///
    /// Providing incorrect index will lead to UB
    pub(crate) fn swap_remove_dense(&mut self, index: usize) -> T {
        let old_dense_len = self.dense_len;

        // move the element from the storage to the local variable
        let value = unsafe { std::ptr::read(self.dense_values_start_ptr.add(index)) };
        // key doesn't need to be explicitly dropped

        let last_dense_index = old_dense_len - 1;

        if index < last_dense_index {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    self.dense_values_start_ptr.add(last_dense_index),
                    self.dense_values_start_ptr.add(index),
                    1,
                );
                std::ptr::copy_nonoverlapping(
                    self.dense_keys_start_ptr.add(last_dense_index),
                    self.dense_keys_start_ptr.add(index),
                    1,
                );
            }
        }

        self.dense_len -= 1;

        value
    }

    pub(crate) fn clear_dense(&mut self) {
        // make sure to drop the values, no need to explicitly drop keys
        if self.dense_len > 0 {
            if std::mem::needs_drop::<T>() {
                for i in 0..self.dense_len {
                    unsafe {
                        std::ptr::drop_in_place(self.dense_values_start_ptr.add(i));
                    }
                }
            }

            self.dense_len = 0;
        }
    }

    pub(crate) fn into_dense_values(mut self) -> Vec<T> {
        // we are going to drop the set, so make sure we don't drop the values again
        // after we moved them out
        let count = self.dense_len;
        self.dense_len = 0;

        let mut out = Vec::with_capacity(count);
        let src = self.dense_values_start_ptr;
        unsafe {
            let dst: *mut T = out.as_mut_ptr();
            std::ptr::copy_nonoverlapping(src, dst, count);
            out.set_len(count);
        }
        out
    }

    pub(crate) fn get_dense_values(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.dense_values_start_ptr, self.dense_len) }
    }

    pub(crate) fn get_dense_values_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.dense_values_start_ptr, self.dense_len) }
    }

    pub(crate) fn get_dense_keys(&self) -> &[SparseKey] {
        unsafe { std::slice::from_raw_parts(self.dense_keys_start_ptr, self.dense_len) }
    }

    pub(crate) fn get_dense_keys_mut(&mut self) -> &mut [SparseKey] {
        unsafe { std::slice::from_raw_parts_mut(self.dense_keys_start_ptr, self.dense_len) }
    }

    pub(crate) fn get_sparse(&self) -> &[SparseEntry] {
        unsafe { std::slice::from_raw_parts(self.sparse_start_ptr, self.sparse_len) }
    }

    pub(crate) fn get_sparse_mut(&mut self) -> &mut [SparseEntry] {
        unsafe { std::slice::from_raw_parts_mut(self.sparse_start_ptr, self.sparse_len) }
    }

    pub(crate) fn get_dense_len(&self) -> usize {
        self.dense_len
    }

    pub(crate) fn get_dense_capacity(&self) -> usize {
        self.max_dense_elements
    }

    pub(crate) fn get_sparse_len(&self) -> usize {
        self.sparse_len
    }

    pub(crate) fn reserve(&mut self, additional: usize) {
        if additional == 0 {
            return;
        }

        let old_sparse_len = self.sparse_len;
        let old_dense_len = self.dense_len;
        let desired_capacity = old_sparse_len + additional;
        if let Some(previous_layout) = self.layout {
            // check if we need to reallocate the buffer
            if self.max_sparse_elements < desired_capacity {
                // dense is always equal or less in size than sparse
                // so no need to check for dense_len

                // need to reallocate the buffer
                let new_max_sparse_elements = desired_capacity;
                let exhausted_sparse_elements = old_sparse_len - old_dense_len;
                let new_max_dense_elements = new_max_sparse_elements - exhausted_sparse_elements;

                let (layout, buffer, dense_keys_offset, sparse_offset) = Self::allocate_new_buffer(
                    size_of::<T>(),
                    align_of::<T>(),
                    new_max_dense_elements,
                    new_max_sparse_elements,
                );

                if layout.is_none() || buffer.is_null() {
                    panic!("Failed to allocate memory for the new buffer of SparseArrayStorage");
                };

                let new_dense_values_start_ptr = buffer as *mut T;
                let new_dense_keys_start_ptr =
                    unsafe { buffer.add(dense_keys_offset) as *mut SparseKey };
                let new_sparse_start_ptr = unsafe { buffer.add(sparse_offset) as *mut SparseEntry };

                // copy the old values
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        self.dense_values_start_ptr,
                        new_dense_values_start_ptr,
                        old_dense_len,
                    );
                    std::ptr::copy_nonoverlapping(
                        self.dense_keys_start_ptr,
                        new_dense_keys_start_ptr,
                        old_dense_len,
                    );
                    std::ptr::copy_nonoverlapping(
                        self.sparse_start_ptr,
                        new_sparse_start_ptr,
                        old_sparse_len,
                    );
                }

                // deallocate the old buffer
                Self::deallocate_buffer(self.buffer, previous_layout);

                self.dense_values_start_ptr = buffer as *mut T;
                self.dense_keys_start_ptr = new_dense_keys_start_ptr;
                self.dense_len = old_dense_len;
                self.sparse_start_ptr = new_sparse_start_ptr;
                self.sparse_len = old_sparse_len;

                self.max_dense_elements = new_max_dense_elements;
                self.max_sparse_elements = new_max_sparse_elements;

                self.buffer = buffer;
                self.layout = layout;
            }
        } else {
            // we never allocated the buffer before
            let (layout, buffer, dense_keys_offset, sparse_offset) = Self::allocate_new_buffer(
                size_of::<T>(),
                align_of::<T>(),
                desired_capacity,
                desired_capacity,
            );

            self.dense_values_start_ptr = buffer as *mut T;
            self.dense_keys_start_ptr = unsafe { buffer.add(dense_keys_offset) as *mut SparseKey };
            self.dense_len = 0;
            self.sparse_start_ptr = unsafe { buffer.add(sparse_offset) as *mut SparseEntry };
            self.sparse_len = 0;

            self.max_dense_elements = desired_capacity;
            self.max_sparse_elements = desired_capacity;

            self.buffer = buffer;
            self.layout = layout;
        }
    }

    fn shift_dense_values_to_the_right(
        &mut self,
        start_index: usize,
        end_index: usize,
        shift_by: usize,
    ) {
        unsafe {
            std::ptr::copy(
                self.dense_values_start_ptr.add(start_index),
                self.dense_values_start_ptr.add(start_index + shift_by),
                end_index - start_index,
            );
            std::ptr::copy(
                self.dense_keys_start_ptr.add(start_index),
                self.dense_keys_start_ptr.add(start_index + shift_by),
                end_index - start_index,
            );
        }
    }

    fn allocate_new_buffer(
        size_of_value: usize,
        align_of_value: usize,
        new_max_dense_values: usize,
        new_max_sparse_values: usize,
    ) -> (Option<std::alloc::Layout>, *mut u8, usize, usize) {
        const SIZE_OF_DENSE_KEY: usize = size_of::<SparseKey>();
        const SIZE_OF_SPARSE_ENTRY: usize = size_of::<SparseEntry>();

        const ALIGN_OF_DENSE_KEY: usize = align_of::<SparseKey>();
        const ALIGN_OF_SPARSE_ENTRY: usize = align_of::<SparseEntry>();

        // for the simplicity sake, we take the largest alignment
        // we could theoretically go with the alignment of the first element,
        // but that would require calculating the paddings based on runtime value of the pointer
        let align_of_buffer: usize = align_of_value
            .max(ALIGN_OF_DENSE_KEY)
            .max(ALIGN_OF_SPARSE_ENTRY);
        let values_end = size_of_value * new_max_dense_values;

        let value_size_reminder = values_end % ALIGN_OF_DENSE_KEY;
        let dense_keys_offset = values_end
            + (ALIGN_OF_DENSE_KEY - value_size_reminder) * (value_size_reminder != 0) as usize;

        let dense_keys_end = dense_keys_offset + SIZE_OF_DENSE_KEY * new_max_dense_values;

        let dense_keys_size_reminder = dense_keys_end % ALIGN_OF_SPARSE_ENTRY;
        let sparse_offset = dense_keys_end
            + (ALIGN_OF_SPARSE_ENTRY - dense_keys_size_reminder)
                * (dense_keys_size_reminder != 0) as usize;

        let sparse_end = sparse_offset + SIZE_OF_SPARSE_ENTRY * new_max_sparse_values;
        let buffer_size_reminder = sparse_end % align_of_buffer;
        // the buffer size should be a multiple of the alignment
        let size_of_buffer = sparse_end
            + (align_of_buffer - buffer_size_reminder) * (buffer_size_reminder != 0) as usize;

        let layout: Option<std::alloc::Layout>;
        let mut buffer: *mut u8 = std::ptr::null_mut();
        unsafe {
            layout = std::alloc::Layout::from_size_align(size_of_buffer, align_of_buffer).ok();
            if let Some(layout) = layout {
                buffer = std::alloc::alloc(layout);
            }
            assert!(
                !buffer.is_null(),
                "Failed to allocate memory for SparseArrayStorage"
            );
        }

        (layout, buffer, dense_keys_offset, sparse_offset)
    }

    fn deallocate_buffer(raw: *mut u8, layout: std::alloc::Layout) {
        unsafe {
            std::alloc::dealloc(raw, layout);
        }
    }
}

impl<T> Clone for SparseArrayStorage<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let (layout, buffer, dense_keys_offset, sparse_offset) = Self::allocate_new_buffer(
            size_of::<T>(),
            align_of::<T>(),
            self.max_dense_elements,
            self.max_sparse_elements,
        );

        let new_dense_values_start_ptr = buffer as *mut T;
        let new_dense_keys_start_ptr = unsafe { buffer.add(dense_keys_offset) as *mut SparseKey };
        let new_sparse_start_ptr = unsafe { buffer.add(sparse_offset) as *mut SparseEntry };

        unsafe {
            // copy by invoking clone on the elements that don't have Copy trait
            if std::mem::needs_drop::<T>() {
                for i in 0..self.dense_len {
                    std::ptr::write(
                        new_dense_values_start_ptr.add(i),
                        (*self.dense_values_start_ptr.add(i)).clone(),
                    );
                }
            } else {
                std::ptr::copy_nonoverlapping(
                    self.dense_values_start_ptr,
                    new_dense_values_start_ptr,
                    self.dense_len,
                );
            }

            std::ptr::copy_nonoverlapping(
                self.dense_keys_start_ptr,
                new_dense_keys_start_ptr,
                self.dense_len,
            );
            std::ptr::copy_nonoverlapping(
                self.sparse_start_ptr,
                new_sparse_start_ptr,
                self.sparse_len,
            );
        }

        Self {
            dense_values_start_ptr: new_dense_values_start_ptr,
            dense_keys_start_ptr: new_dense_keys_start_ptr,
            dense_len: self.dense_len,
            sparse_start_ptr: new_sparse_start_ptr,
            sparse_len: self.sparse_len,

            max_dense_elements: self.max_dense_elements,
            max_sparse_elements: self.max_sparse_elements,

            buffer,
            layout,
        }
    }
}

impl<T> Drop for SparseArrayStorage<T> {
    fn drop(&mut self) {
        if let Some(layout) = self.layout {
            self.clear_dense();
            Self::deallocate_buffer(self.buffer, layout);
        }
    }
}

unsafe impl<T: Send> Send for SparseArrayStorage<T> {}
unsafe impl<T: Sync> Sync for SparseArrayStorage<T> {}
