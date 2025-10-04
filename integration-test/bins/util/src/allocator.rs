use good_memory_allocator::SpinLockedAllocator;

#[repr(align(0x4000))]
struct Align16K<T>(T);

/// 16 KiB naturally aligned backing storage for heap.
static mut HEAP: Align16K<[u8; 0x4000]> = Align16K([0; 0x4000]);

#[global_allocator]
static ALLOCATOR: SpinLockedAllocator = SpinLockedAllocator::empty();

/// Initializes the allocator. Call only once.
pub fn init() {
    #[allow(static_mut_refs)]
    unsafe {
        ALLOCATOR.init(HEAP.0.as_ptr().cast::<usize>() as _, HEAP.0.len());
    }
}
