use good_memory_allocator::SpinLockedAllocator;

#[repr(align(0x1000))]
struct PageAlign<T>(T);

/// 16 KiB page-aligned backing storage for heap.
static mut HEAP: PageAlign<[u8; 0x4000]> = PageAlign([0; 0x4000]);

#[global_allocator]
static ALLOCATOR: SpinLockedAllocator = SpinLockedAllocator::empty();

/// Initializes the allocator. Call only once.
pub fn init() {
    unsafe {
        ALLOCATOR.init(HEAP.0.as_ptr().cast::<usize>() as _, HEAP.0.len());
    }
}
