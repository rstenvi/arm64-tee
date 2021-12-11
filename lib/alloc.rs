use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

use lib::math;
use lib::bitmap;
use lib::memory;
use driver::mmu;
use applets;
use cpu;
use applets::arch_svc;


extern "C" {
	fn memset(addr: u64, c: i8, size: u64);
}

pub struct Allocator { }

const MEMORY_REGION: u64 = mmu::maxmem_upper!() - (mmu::PAGE_SIZE * 2);
const MALLOC_REGION: u64 = mmu::maxmem_upper!() - (mmu::PAGE_SIZE * 256);

const BLOCK_INC:  u64 = 4;
const BLOCK_SIZE: u32 = 16;

const MAX_PAGES:  u64 = 128;


impl Allocator {
	pub const fn empty() -> Allocator {
		Allocator { }
    }
	pub fn init(&self) -> i32 {
		// Allocate region to store bitmap
		let meta = applets::svc!(cpu::svc::SYSNO_MMAP, MEMORY_REGION, mmu::PAGE_SIZE, mmu::EL0_RW);
		assert!(meta == MEMORY_REGION);

		// Allocate memory for data region
		let addr = applets::svc!(
			cpu::svc::SYSNO_MMAP, MALLOC_REGION, BLOCK_INC * mmu::PAGE_SIZE, mmu::EL0_RW
		);
		assert!(addr == MALLOC_REGION);

		// Mark entire bitmap as free
		unsafe { memset(meta, 0x00, mmu::PAGE_SIZE) };

		// Allocate one block to store our own data
		// We should get the first block
		assert!(bitmap::alloc(meta, mmu::PAGE_SIZE, 1) == 0);

		// Store the number of blocks we've allocated for data
		memory::dma::write::u64(MALLOC_REGION, BLOCK_INC);

		return 0;
	}
	pub fn ensure_space(&self, blocks: u64) -> i32 {
		if blocks > MAX_PAGES {
			return -1;
		}
		let cblks = memory::dma::read::u64(MALLOC_REGION);
		if cblks < blocks {
			let nblocks = blocks - cblks;
			let nsaddr = MALLOC_REGION + (cblks * mmu::PAGE_SIZE);
			let addr = applets::svc!(
				cpu::svc::SYSNO_MMAP, nsaddr, nblocks * mmu::PAGE_SIZE, mmu::EL0_RW
			);
			assert!(addr == nsaddr);

			// Write back number of blocks we now have
			memory::dma::write::u64(MALLOC_REGION, blocks);
		}
		return 0;
	}
}

unsafe impl GlobalAlloc for Allocator {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		let size = math::align_pow2_up!(layout.size() as u32, BLOCK_SIZE) as u32;
		let blocks = size / BLOCK_SIZE;


		let idx = bitmap::alloc(MEMORY_REGION, mmu::PAGE_SIZE, blocks as u64);
		if idx == u64::MAX {
			return ptr::null_mut();
		}
		let doffset = idx * BLOCK_SIZE as u64;
		self.ensure_space(math::align_pow2_up!(doffset, mmu::PAGE_SIZE) / mmu::PAGE_SIZE);

		let addr = MALLOC_REGION + doffset;

 		return addr as *mut u8;
	}
	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		let size = math::align_pow2_up!(layout.size() as u32, BLOCK_SIZE) as u32;
		let rptr = ptr as u64;
		let offset = rptr - MALLOC_REGION;
		let blocks = size / BLOCK_SIZE;
		let startidx = offset / BLOCK_SIZE as u64;
		
		bitmap::free(MEMORY_REGION, mmu::PAGE_SIZE, startidx, blocks as u64);
	}
}


#[global_allocator]
static ALLOCATOR: Allocator = Allocator::empty();

pub fn init() -> i32 {
	return ALLOCATOR.init();
}
