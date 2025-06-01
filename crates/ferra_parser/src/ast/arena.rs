//! Arena allocator for AST nodes
//!
//! Uses bumpalo for efficient allocation of AST nodes without individual deallocations

use bumpalo::Bump;

/// Arena for allocating AST nodes efficiently
pub struct Arena {
    bump: Bump,
}

impl Arena {
    /// Create a new arena
    pub fn new() -> Self {
        Self { bump: Bump::new() }
    }

    /// Allocate a value in the arena and return a reference
    pub fn alloc<T>(&self, value: T) -> &T {
        self.bump.alloc(value)
    }

    /// Allocate a slice in the arena
    pub fn alloc_slice<T>(&self, slice: &[T]) -> &[T]
    where
        T: Clone,
    {
        self.bump.alloc_slice_clone(slice)
    }

    /// Allocate a vector in the arena
    pub fn alloc_vec<T>(&self, vec: Vec<T>) -> &[T]
    where
        T: Clone,
    {
        self.bump.alloc_slice_clone(&vec)
    }

    /// Get the number of bytes allocated in this arena
    pub fn allocated_bytes(&self) -> usize {
        self.bump.allocated_bytes()
    }

    /// Reset the arena, deallocating all stored values
    pub fn reset(&mut self) {
        self.bump.reset();
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_allocation() {
        let arena = Arena::new();

        let value = arena.alloc(42i32);
        assert_eq!(*value, 42);

        let string = arena.alloc("hello".to_string());
        assert_eq!(string, "hello");
    }

    #[test]
    fn test_arena_slice_allocation() {
        let arena = Arena::new();

        let slice = arena.alloc_slice(&[1, 2, 3, 4]);
        assert_eq!(slice, &[1, 2, 3, 4]);
    }

    #[test]
    fn test_arena_reset() {
        let mut arena = Arena::new();

        // Allocate something
        let _value = arena.alloc(100);
        let _bytes_before = arena.allocated_bytes();

        // Reset should clear allocations
        arena.reset();
        let _bytes_after = arena.allocated_bytes();

        // After reset, we should be able to allocate again
        // (bumpalo might keep capacity, so we just test functionality)
        let value2 = arena.alloc(200);
        assert_eq!(*value2, 200);
    }
}
