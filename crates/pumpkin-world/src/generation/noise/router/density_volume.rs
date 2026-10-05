use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

pub use pumpkin_util::noise::volume::DensityVolume;

const BUFFER_SIZE_INCREMENT: usize = 16;
const MAX_REUSE_SIZE_FACTOR: usize = 2;
const MAX_POOLED_BUFFERS: usize = 1024;

thread_local! {
    static DENSITY_BUFFER_POOL: RefCell<Vec<Box<[f64]>>> = const { RefCell::new(Vec::new()) };
}

fn take_best(
    pool: &mut Vec<Box<[f64]>>,
    min_capacity: usize,
    max_capacity: usize,
) -> Option<Box<[f64]>> {
    let mut best_index = None;
    let mut best_capacity = max_capacity + 1;
    for i in (0..pool.len()).rev() {
        let capacity = pool[i].len();
        if capacity == min_capacity {
            return Some(pool.remove(i));
        }
        if capacity > min_capacity && capacity < best_capacity {
            best_index = Some(i);
            best_capacity = capacity;
        }
    }
    best_index.map(|i| pool.remove(i))
}

pub struct DensityBuffer {
    values: Box<[f64]>,
    len: usize,
}

impl DensityBuffer {
    #[must_use]
    pub fn acquire(volume: &DensityVolume) -> Self {
        Self::with_len(volume.size())
    }

    #[must_use]
    pub fn with_len(len: usize) -> Self {
        let min_capacity = len.next_multiple_of(BUFFER_SIZE_INCREMENT);
        let values = DENSITY_BUFFER_POOL
            .with(|pool| {
                take_best(
                    &mut pool.borrow_mut(),
                    min_capacity,
                    min_capacity * MAX_REUSE_SIZE_FACTOR,
                )
            })
            .unwrap_or_else(|| vec![0.0; min_capacity].into_boxed_slice());
        Self { values, len }
    }

    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.values.len()
    }
}

impl Deref for DensityBuffer {
    type Target = [f64];

    #[inline]
    fn deref(&self) -> &[f64] {
        &self.values[..self.len]
    }
}

impl DerefMut for DensityBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut [f64] {
        &mut self.values[..self.len]
    }
}

impl Drop for DensityBuffer {
    fn drop(&mut self) {
        let values = std::mem::take(&mut self.values);
        DENSITY_BUFFER_POOL.with(|pool| {
            let mut pool = pool.borrow_mut();
            if pool.len() < MAX_POOLED_BUFFERS {
                pool.push(values);
            }
        });
    }
}
