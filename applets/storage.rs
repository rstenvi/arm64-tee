use applets;
use applets::arch_svc;
use driver::mmu;
use lib::math;
use cpu;
use alloc::boxed::Box;
use lib::alloc;
use core::borrow::Borrow;
// use core::mem;
// use alloc::vec::Vec;

// #[link_section = ".userdata.storage"]
// static mut GVAR: i32 = 0;


const MEMORY_REGION: u64 = mmu::maxmem_upper!() - (mmu::PAGE_SIZE * 128);

pub fn init() {
	alloc::init();
	let boxed: Box<u64> = Box::new(42);
	let ptr = Box::into_raw(boxed);
	applets::svc!(cpu::svc::SYSNO_STORE_PTR, ptr as u64);
// 	let mut xs = Vec::new();
// 	xs.push(1);

	applets::svc!(cpu::svc::SYSNO_EXIT);
}

pub fn smc(data: u64, func: u64, _cmd: u64, _arg: u64, _len: u64)	{
	let x = unsafe { Box::from_raw(data as *mut u64) };
	let val = x.borrow();
	applets::svc!(cpu::svc::SYSNO_EXIT, func, *val);
}
