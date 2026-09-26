#![warn(clippy::all)]

pub mod app;
pub mod cursor;
pub mod grid;
pub mod midi;
pub mod native_midi;
pub mod opts;
pub mod playback;
pub mod region;
pub mod render_frame;
pub mod source;

#[cfg(test)]
mod test {
    use std::sync::Once;

    static INIT: Once = Once::new();

    pub fn trace() {
        INIT.call_once(|| {
            tracing_subscriber::fmt()
                .with_env_filter("debug")
                .with_file(true)
                .with_line_number(true)
                .pretty()
                .init();
        });
    }
}

/// Allocation counting for this crate's unit tests, which measure derivations
/// no public API reaches in isolation.
///
/// The allocator is `tests/allocation.rs`'s block counting, without its byte
/// count, installed in the unit-test binary because a `#[global_allocator]` is
/// per binary. It is one harness with `tests/allocation.rs` and
/// `lang/tests/allocation.rs`: a change to the counting belongs in all three.
#[cfg(all(test, not(target_arch = "wasm32")))]
pub(crate) mod allocation {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::Cell;

    thread_local! {
        // `const` initialisation is what makes this safe to touch from inside
        // the allocator: a lazily initialised thread-local allocates on first
        // access, which would be an allocation from within the allocation it
        // is counting. `Cell<usize>` has no drop glue, so no destructor is
        // registered either.
        static BLOCKS: Cell<usize> = const { Cell::new(0) };
    }

    /// `System`, plus a count of the blocks handed out on each thread.
    ///
    /// `realloc` and `alloc_zeroed` are not overridden: `GlobalAlloc`'s
    /// defaults implement both through `Self::alloc` and `Self::dealloc`, so
    /// they are counted without the counting being written twice.
    struct Counting;

    // SAFETY: `GlobalAlloc`'s contract is discharged entirely by `System`,
    // which already upholds it. Every method forwards to `System` with the very
    // layout and pointer it was handed and returns exactly what `System`
    // returned, including a null pointer on failure. So every block handed out
    // was allocated by `System`, and every block freed is passed to `System`
    // with the layout it was allocated with. The counter is a thread-local
    // integer that touches neither the pointer nor the layout, and its
    // `const`-initialised storage cannot allocate and so cannot re-enter this
    // allocator.
    unsafe impl GlobalAlloc for Counting {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            BLOCKS.with(|blocks| blocks.set(blocks.get() + 1));
            // SAFETY: `layout` is the caller's, forwarded unchanged, and the
            // caller has already met `alloc`'s requirement that it be non-zero
            // sized. The result is returned unchanged, so `System` owns it.
            unsafe { System.alloc(layout) }
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            // SAFETY: `ptr` came from `System.alloc` above, and `layout` is
            // the one it was allocated with, because both are forwarded
            // unchanged in both directions.
            unsafe { System.dealloc(ptr, layout) }
        }
    }

    #[global_allocator]
    static ALLOCATOR: Counting = Counting;

    /// Runs `f` and answers how many blocks it allocated on this thread,
    /// alongside what it produced. The value is handed back rather than
    /// dropped inside, so a freeing `Drop` falls outside the count.
    pub(crate) fn blocks<T>(f: impl FnOnce() -> T) -> (usize, T) {
        let before = BLOCKS.with(Cell::get);
        let value = f();
        (BLOCKS.with(Cell::get) - before, value)
    }
}
