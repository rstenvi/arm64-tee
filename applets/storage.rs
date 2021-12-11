use applets;
use applets::arch_svc;
use driver::mmu;
use lib::math;
use lib::alloc;
use cpu;

// extern crate alloc;
use alloc::vec::Vec;

const MEMORY_REGION: u64 = mmu::maxmem_upper!() - (mmu::PAGE_SIZE * 128);

pub fn init() {
	alloc::init();

	let mut xs = Vec::new();
	xs.push(1);

	applets::svc!(cpu::svc::SYSNO_EXIT);
}

pub fn smc(func: u64, _cmd: u64, _arg: u64, _len: u64)	{

	
	applets::svc!(cpu::svc::SYSNO_EXIT, func, 0);
}
