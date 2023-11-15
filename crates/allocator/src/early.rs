use spinlock::SpinNoIrq;
use core::{ptr::NonNull, alloc::Layout};
use crate::{ByteAllocator, AllocError, BaseAllocator,AllocResult, PageAllocator};
//const MIN_HEAP_SIZE: usize = 0x8000; // 32 K
//const PAGE_SIZE: usize = 0x1000;
pub struct EarlyByteAllocator {
    start: usize,
    pos: usize,
    total_bytes: usize,
    used_bytes: usize,
}
pub struct EarlyPageAllocator<const PAGE_SIZE: usize> {
    end: usize,
    pos: usize,
    total_pages: usize,
    used_pages: usize,
}
/// EarlyAllocator
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    balloc: SpinNoIrq<EarlyByteAllocator>,
    palloc: SpinNoIrq<EarlyPageAllocator<PAGE_SIZE>>,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    /// Creates a new empty `EarlyAllocator`.
    pub const fn new() -> Self {
        Self {
            balloc: SpinNoIrq::new(EarlyByteAllocator::new()),
            palloc: SpinNoIrq::new(EarlyPageAllocator::new()),
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
     fn init(&mut self, start_vaddr: usize, size: usize) {
        self.palloc.lock().init(start_vaddr, size);
        self.balloc.lock().init(start_vaddr, size);
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        self.balloc.lock().add_memory(start, size)
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        self.balloc.lock().alloc(layout)
    }

    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        self.balloc.lock().dealloc(pos, layout)
    }

    fn total_bytes(&self) -> usize {
        self.balloc.lock().total_bytes()
    }

    fn used_bytes(&self) -> usize {
        self.balloc.lock().used_bytes()
    }

    fn available_bytes(&self) -> usize {
        self.balloc.lock().available_bytes()
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        self.palloc.lock().alloc_pages(num_pages, align_pow2)
    }

    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        self.palloc.lock().dealloc_pages(pos, num_pages)
    }

    fn total_pages(&self) -> usize {
        self.palloc.lock().total_pages()
    }

    fn used_pages(&self) -> usize {
        self.palloc.lock().used_pages()
    }

    fn available_pages(&self) -> usize {
        self.palloc.lock().available_pages()
    }
}

///early_byte

impl EarlyByteAllocator {
    pub const fn new() -> Self {
        Self {
            start: 0,
            pos: 0,
            total_bytes: 0,
            used_bytes: 0,
        }
    }

    #[inline]
    fn increase_pos(&mut self, size: usize) {
        self.pos += size;
    }

    #[inline]
    fn reset_pos(&mut self) {
        if self.used_bytes == 0 {
            self.pos = self.start;
        }
    }
}

impl BaseAllocator for EarlyByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.pos = start;
        self.total_bytes = size;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> crate::AllocResult {
        Err(AllocError::NoMemory) // unsupported
    }
}

impl ByteAllocator for EarlyByteAllocator {
    fn alloc(&mut self, layout: Layout) -> crate::AllocResult<NonNull<u8>> {
        match NonNull::new(self.pos as *mut u8) {
            Some(pos) => {
                let size = layout.size();
                self.used_bytes += size;
                self.increase_pos(size);
                Ok(pos)
            }
            None => {
                Err(AllocError::NoMemory)
            }
        }
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, layout: Layout) {
        let size = layout.size();
        self.used_bytes -= size;
        self.reset_pos();
    }

    fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    fn used_bytes(&self) -> usize {
        self.used_bytes
    }

    fn available_bytes(&self) -> usize {
        self.total_bytes - self.used_bytes
    }
}
///early_page
impl<const PAGE_SIZE: usize> EarlyPageAllocator<PAGE_SIZE> {
    /// Creates a new empty `EarlyPageAllocator`.
    pub const fn new() -> Self {
        Self {
            end: 0,
            pos: 0,
            total_pages: 0,
            used_pages: 0,
        }
    }

    #[inline]
    pub fn decrease_pos(&mut self, num_pages: usize) {
        self.pos -= PAGE_SIZE * num_pages;
    }

    #[inline]
    pub fn reset_pos(&mut self) {
        if self.used_pages == 0 {
            self.pos = self.end;
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyPageAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        assert!(PAGE_SIZE.is_power_of_two());

        self.end = super::align_down(start + size, PAGE_SIZE);
        let start = super::align_up(start, PAGE_SIZE);
        self.pos = self.end;
        self.total_pages = (self.end - start) / PAGE_SIZE;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        Err(AllocError::NoMemory) // unsupported
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyPageAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        if align_pow2 % PAGE_SIZE != 0 {
            return Err(AllocError::InvalidParam);
        }

        let align_pow2 = align_pow2 / PAGE_SIZE;
        if !align_pow2.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }

        match num_pages.cmp(&1) {
            core::cmp::Ordering::Equal => Some(self.pos - PAGE_SIZE),
            core::cmp::Ordering::Greater => Some(self.pos - PAGE_SIZE * num_pages),
            _ => return Err(AllocError::InvalidParam),
        }
        .ok_or(AllocError::NoMemory)
        .inspect(|_| {
            self.used_pages += num_pages;

            self.decrease_pos(num_pages);
        })
    }

    fn dealloc_pages(&mut self, _pos: usize, num_pages: usize) {
        // TODO: not decrease `used_pages` if deallocation failed
        self.used_pages -= num_pages;
        self.reset_pos();
    }

    fn total_pages(&self) -> usize {
        self.total_pages
    }

    fn used_pages(&self) -> usize {
        self.used_pages
    }

    fn available_pages(&self) -> usize {
        self.total_pages - self.used_pages
    }
}