#[derive(Copy, Clone)]
pub(crate) enum SparseEntry {
    AliveEntry(AliveSparseEntry),
    FreeEntry(FreeSparseEntry),
}

#[derive(Copy, Clone)]
pub(crate) struct AliveSparseEntry {
    pub(crate) dense_index: usize,
    pub(crate) epoch: usize,
}

#[derive(Copy, Clone)]
pub(crate) struct FreeSparseEntry {
    pub(crate) next_free: usize,
    pub(crate) next_epoch: usize,
}
