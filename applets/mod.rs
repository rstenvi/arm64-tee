/**
* Module controlling all user-mode applets.
*
* This is basically a simple thread manager without any pre-emptive scheduling.
* The main purpose is to provide isolation between applets. This isolation is
* accomplished by using a separate page directory for all data segments.
*/
use cpu::interrupt;
use driver::mmu;
use lib::math;
use cpu;

mod storage;

extern "C" {
	fn drop_el0(x: &interrupt::InterruptState);
	pub fn arch_svc(a0: u64, a1: u64, a2: u64, fnid: u64) -> u64;
	fn arch_load_offset() -> u64;
}

#[macro_export]
macro_rules! svc {
	($id:expr) => {
		unsafe { arch_svc(0, 0, 0, $id) }
	};
	($id:expr, $a1:expr) => {
		unsafe { arch_svc($a1, 0, 0, $id) }
	};
	($id:expr, $a1:expr, $a2:expr) => {
		unsafe { arch_svc($a1, $a2, 0, $id) }
	};
	($id:expr, $a1:expr, $a2:expr, $a3:expr) => {
		unsafe { arch_svc($a1, $a2, $a3, $id) }
	};
}
macro_rules! adjust_func_add {
	($addr:expr) => {
//		let offset = unsafe { arch_load_offset() };
		$addr + unsafe { arch_load_offset() }
	};
}
macro_rules! exec_in_el0 {
	($app:expr) => {
		let mut state = interrupt::InterruptState::default();
		state.elr = adjust_func_add!($app.init as u64);
		__exec_in_el0($app, &mut state);
	};
	($app:expr, $fnid:expr, $cmd:expr, $arg:expr) => {
		let mut state = interrupt::InterruptState::default();
		state.elr = adjust_func_add!($app.smc as u64);
		state.regs[0] = $fnid;
		state.regs[1] = $cmd;
		state.regs[2] = $arg;
		__exec_in_el0($app, &mut state);
	};
	($app:expr, $fnid:expr, $cmd:expr, $arg:expr, $len:expr) => {
		let mut state = interrupt::InterruptState::default();
		state.elr = adjust_func_add!($app.smc as u64);
		state.regs[0] = $fnid;
		state.regs[1] = $cmd;
		state.regs[2] = $arg;
		state.regs[2] = $len;
		__exec_in_el0($app, &mut state);
	};
}

// macro_rules! cmdid {
// 	($magic:expr, $num:expr, $argt:expr, $size:expr) => {
// 		$magic << 24 | $num << 16 | $argt << 14 | $size
// 	};
// 	($magic:expr, $num:expr) => {
// 		cmdid!($expr, $num, 0, 0)
// 	};
// }
// macro_rules! cmdid_to_type {
// 	($id:expr) => {
// 		(($id >> 14) & 0b11), ($id & ((1<<14)-1))
// 	};
// }

pub(crate) use svc;

type AppletSmc  = fn(u64, u64, u64, u64);
type AppletInit = fn();

/**
* All the information we need about each applet.
*
* The applet has access to all code running on the system, but has no .data,
* .rodata or .bss segment, i.e. it has no global variables. This could
* potentially be fixed by the applet creating data in custom sections with
*
* #[link_section = ".applet.name"]
*
* We could then parse the section name to figure out which regions should be
* mapped in for applet. It would be a bit awkward to integrate that logic with
* the linker, but should be doable.
*
* Current method for the applet is to allocate memory dynamically with the
* SYSNO_MMAP call at any VA in the upper half where it executes. During the
* init-function the applet has an oppurtunity to set all this up.
*/
#[derive(Debug)]
pub struct Applet {
	ttbr: u64,
	stack: u64,
	init: AppletInit,
	smc:  AppletSmc,
	ready: bool,
}

/*
* Declare all registered applets
*/
static mut APPLETS: [Applet; 1] = [
	Applet{smc: storage::smc, init: storage::init, stack: u64::MAX, ttbr: u64::MAX, ready: false}
];

// Empty function if applet doesn't need any initialization
// Is ignored below and not called, but if called, it would be valid
fn init_el0_empty() { svc!(cpu::svc::SYSNO_EXIT); }

pub fn __exec_in_el0(app: &Applet, state: &mut interrupt::InterruptState)	{
	// See D1.6.4 Saved Program Status Registers (SPSRs)
	state.spsr = 0;
	state.saved_sp = app.stack;

	mmu::switch_ttbr1(app.ttbr);
	unsafe { drop_el0(&state) };
}

fn init_session(app: &mut Applet) -> i32 {
	if app.ttbr == u64::MAX {
		let ttbr = mmu::alloc_pgd();

		// Allocate stack at highest available memory
		let stackva = mmu::maxmem_upper!() - mmu::PAGE_SIZE;
		mmu::alloc_page(ttbr, stackva, mmu::EL0_RW);
		app.stack = stackva + mmu::PAGE_SIZE;
		app.ttbr = ttbr;
	}
	return 0;
}

fn exec_smc(idx: usize, fnid: u64, cmd: u64, mut arg: u64, len: u64) -> i32 {
	let mlen = unsafe { APPLETS.len() };
	if idx < mlen {
		let app = unsafe { &mut APPLETS[idx] };
		init_session(app);
		if len > 0 {
			mmu::memcpy_ns(app.ttbr, mmu::VA_RESERVED_START, arg, len, true);
			arg = mmu::VA_RESERVED_START;
		}
		exec_in_el0!(app, fnid, cmd, arg, len);
		return 0;
	}
	return -1;
}

pub fn smc_handler(func: u64, args: &mut [u64; 8]) {
	// func is 64b but only lowest 16b can be set
	let ret = exec_smc(func as usize, args[0], args[1], args[2], args[3]);

	if ret < 0 {
		// Keep args[0] intact since it's the function id
		args[1] = u64::MAX;
		args[2] = 0;
		args[3] = 0;
	} else {
		args[1] = ret as u64;
	}
}

pub fn init() -> u32 {
	let len = unsafe { APPLETS.len() };
	for i in 0..len {
		let app = unsafe { &mut APPLETS[i] };
		if ! app.ready {
			app.ready = true;
			if app.init != init_el0_empty {
				init_session(app);
				exec_in_el0!(app);
			}
		}
	}
	return 0;
}

// Can be used to place data in specific sections
// #[link_section = ".usermode"]

