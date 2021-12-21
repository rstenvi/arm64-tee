#![feature(panic_info_message)]
#![feature(asm)]

// What must be provided for std environment
// <https://docs.rust-embedded.org/book/intro/no-std.html>
#![no_std]
#![crate_name="bl32"]
#![allow(dead_code)]

#![feature(alloc_error_handler)]

extern crate alloc;

mod lib;
mod cpu;
mod driver;
mod platform;
mod applets;
mod syscall;

use lib::log;
// use platform::Platform;
use driver::mmu;
use lib::math;
use core::alloc::Layout;

extern "C" {
	fn switch_stack(s: u64);
}

#[no_mangle]
pub extern "C" fn get_new_stack() -> u64 {
	let mut nstack = mmu::maxmem_lower!() - (mmu::PAGE_SIZE * 2);
	while ! mmu::page_available_el1(nstack) {
		// Reserve some space for overruns
		nstack -= mmu::PAGE_SIZE * 4;
	}
	assert!(mmu::alloc_page_el1(nstack, mmu::EL1_RW) == 0);
	return nstack + mmu::PAGE_SIZE;
}

#[no_mangle]
pub extern "C" fn rustmain(fdt: u64, imgload: u64, imgend: u64) -> u64 {
	log::info("Reached rustmain");

	// FDT is not located in secure memory, so should either copy it over or not
	// use it after the init-routines are done
	let _fdtsize = driver::dtb::init(fdt);

	// Get address of serial device
	let (mut plstart, mut plsize) = driver::dtb::get_as_reg("/pl011/reg", 0);
	if plstart == u64::MAX {
		// Get the default one we've already been using
		log::info("Unable to find address for serial device in DTB");
		plstart = platform::uart::BASE0;
		plsize = mmu::PAGE_SIZE;
	}
	driver::serial::set_base(plstart);

	// Init physical memory manager
	driver::pmm::init(imgload, imgend);

	// Init virtual memory:
	// - Identity map image region
	// - set up linear region for future modifications
	let linear = driver::mmu::init(imgload, imgend);

	// PMM must adjust any dynamically allocated data
	driver::pmm::lateinit(linear);

	// Set up DMA for serial device
	mmu::map_dma(plstart, plstart + plsize);
// 	platform::Impl::map_dma();
	log::info("Initialized virtual memory");

	// Set up new stack unique to CPU core
	let nstack = get_new_stack();
	unsafe { switch_stack(nstack) };

	// Take all registered applets and call the init function if one is defined
	log::info("Starting init of applets");
	applets::init();

	// We will only return here if no applets need initialization
	return 0;
}

#[panic_handler]
pub fn panic_implementation(_info: &::core::panic::PanicInfo) -> ! {
    loop { }
}
#[no_mangle]
pub fn panic() -> ! {
	loop { }
}
#[alloc_error_handler]
pub fn alloc_panic(_layout: Layout) -> ! {
	panic();
}
