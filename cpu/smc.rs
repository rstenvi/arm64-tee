use log;
use applets;
use driver::mmu;

extern "C" {
	fn return_el3();
}


//const SVCID: u64 = 0x32;

/* If bit `SMC_TYPE_FAST << FUNCID_TYPE_SHIFT` is set, it's a fast interrupt */
const SMC_TYPE_YIELD:    u64 = 0;
const SMC_TYPE_FAST:     u64 = 1;
const FUNCID_TYPE_SHIFT: u64 = 31;

/* if `SMC_32 << FUNCID_CC_SHIFT` is set, it's a smc32 call, otherwise smc64 */
const SMC_32:            u64 = 0;
const SMC_64:            u64 = 1;
const FUNCID_CC_SHIFT:   u64 = 30;

/* Starting from bit 24-29, the service number is specified */
const FUNCID_OEN_SHIFT:  u64 = 24;
const FUNCID_OEN_BITS:   u64 = 6;
const FUNCID_OEN_MASK:   u64 = (1 << FUNCID_OEN_BITS) - 1;

/* The lowest 16 bits can be used to specify function number */
const FUNCID_NUM_MASK:   u64 = 0xffff;

macro_rules! SHIFT        { ($var:expr, $shift:expr) => { ($var << $shift) }; }
macro_rules! SMC_BIT_FAST { () => { SHIFT!(SMC_TYPE_FAST, FUNCID_TYPE_SHIFT) }; }
macro_rules! SMC_BIT_32   { () => { SHIFT!(SMC_TYPE_FAST, FUNCID_TYPE_SHIFT) }; }
macro_rules! SMC_OEN      { ($var:expr) => { SHIFT!($var, FUNCID_OEN_SHIFT) }; }
macro_rules! SMC_FNID     { ($var:expr) => { ($var & FUNCID_NUM_MASK) }; }

macro_rules! SMC_ID_FAST32 {
	($fnid:expr, $srvid:expr) => {
		SMC_BIT_FAST!() | SMC_BIT_32!() | SMC_OEN!($fnid) | SMC_FNID!($srvid)
	};
}
macro_rules! SMC_OEN_ID {
	($num:expr) => {
		(($num >> FUNCID_OEN_SHIFT) & FUNCID_OEN_MASK)
	};
}

mod tsp {
	use log;
	pub const SVCID: u64 = 0x32;

	pub const ENTRY_DONE:        u64 = 0xf2000000;
	pub const ON_DONE:           u64 = 0xf2000001;
	pub const OFF_DONE:          u64 = 0xf2000002;
	pub const SUSPEND_DONE:      u64 = 0xf2000003;
	pub const RESUME_DONE:       u64 = 0xf2000004;
	pub const PREEMPTED:         u64 = 0xf2000005;
	pub const ABORT_DONE:        u64 = 0xf2000007;
	pub const SYSTEM_OFF_DONE:   u64 = 0xf2000008;
	pub const SYSTEM_RESET_DONE: u64 = 0xf2000009;

	const FNID_ADD: u64 = 0x2000;
	const FNID_SUB: u64 = 0x2001;
	const FNID_MUL: u64 = 0x2002;
	const FNID_DIV: u64 = 0x2003;

	pub fn smc_handler(func: u64, args: &mut [u64; 8]) -> u64 {
		let mut res = u64::MAX;
		match func {
			FNID_ADD => {
				res = args[1] + args[2];
			}
			FNID_SUB => {
				res = args[1] - args[2];
			}
			FNID_MUL => {
				res = args[1] * args[2];
			}
			FNID_DIV => {
				res = args[1] / args[2];
			}
			_ => {
				log::info("Invalid function called");
			}
		}
		// x4 - x7 MUST be preserved unless they contain arguments
		args[1] = res;
		args[2] = 0;
		args[3] = 0;
		return 0;
	}
	pub fn cpu_off(args: &mut [u64; 8]) {
		args[0] = OFF_DONE;
	}
	pub fn cpu_suspend(args: &mut [u64; 8])	{
		args[0] = SUSPEND_DONE;
	}
	pub fn cpu_resume(args: &mut [u64; 8])	{
		args[0] = RESUME_DONE;
	}
	pub fn system_off(args: &mut [u64; 8])	{
		args[0] = SYSTEM_OFF_DONE;
	}
	pub fn system_reset(args: &mut [u64; 8])	{
		args[0] = SYSTEM_RESET_DONE;
	}
	pub fn cpu_on(args: &mut [u64; 8])	{
		args[0] = ON_DONE;
	}
	pub fn smc_return(_func: u64, _ret: u64) {
		panic!();
	}
}

mod optee {
	extern "C" {
		fn smcret(code: u64, a1: u64, a2: u64, a3: u64);
	}
	use log;
	use cpu::smc::SMC_TYPE_FAST;
	use cpu::smc::FUNCID_TYPE_SHIFT;
	use cpu::smc::FUNCID_NUM_MASK;
	use cpu::smc::FUNCID_OEN_SHIFT;
	//use cpu::smc::FUNCID_CC_SHIFT;
	//use cpu::smc::SMC_32;

	pub const SVCID: u64 = 0x3e;

	const ENTRY_DONE:        u64 = SMC_ID_FAST32!(SVCID, 0x00);
	const ON_DONE:           u64 = SMC_ID_FAST32!(SVCID, 0x01);
	const OFF_DONE:          u64 = SMC_ID_FAST32!(SVCID, 0x02);
	const SUSPEND_DONE:      u64 = SMC_ID_FAST32!(SVCID, 0x03);
	const RESUME_DONE:       u64 = SMC_ID_FAST32!(SVCID, 0x04);
	const CALL_DONE:         u64 = SMC_ID_FAST32!(SVCID, 0x05);
	const FIQ_DONE:          u64 = SMC_ID_FAST32!(SVCID, 0x06);
	const SYSTEM_OFF_DONE:   u64 = SMC_ID_FAST32!(SVCID, 0x07);
	const SYSTEM_RESET_DONE: u64 = SMC_ID_FAST32!(SVCID, 0x08);


	const FNID_LOG: u64 = 1;

	pub fn smc_handler(func: u64, args: &mut [u64; 8]) -> u64 {
		log::info("smc optee called");
		let mut res = u64::MAX;
		match func {
			FNID_LOG => {
				//log::from_memory(args[1]);
				res = 0;
			}
			_ => {
				log::info("optee: invalid function called");
			}
		}
		args[0] = CALL_DONE;
		args[1] = res;
		return 0;
	}
	pub fn smc_return(_func: u64, ret: u64) {
		unsafe { smcret(CALL_DONE, ret, 0, 0); }
	}
	pub fn cpu_off(args: &mut [u64; 8]) {
		args[0] = OFF_DONE;
	}
	pub fn cpu_suspend(args: &mut [u64; 8])	{
		args[0] = SUSPEND_DONE;
	}
	pub fn cpu_resume(args: &mut [u64; 8])	{
		args[0] = RESUME_DONE;
	}
	pub fn system_off(args: &mut [u64; 8])	{
		args[0] = SYSTEM_OFF_DONE;
	}
	pub fn system_reset(args: &mut [u64; 8])	{
		args[0] = SYSTEM_RESET_DONE;
	}
	pub fn cpu_on(args: &mut [u64; 8])	{
		args[0] = ON_DONE;
		args[1] = 0;
	}
	pub fn fiq_entry(args: &mut [u64; 8])	{
		args[0] = FIQ_DONE;
	}
}


/**
* Generic wrapper implementing SMC calling convention
* Full standard: <https://developer.arm.com/documentation/den0028/latest>
*
* Overview of call
* - x0[0:15] contain function id
* - x0[24:29] is service call range, see TF-A
*   - 0x72 is Test Secure Payload
* - x0[30] If set, SMC64 is used
* - x0[31] Fast call is used
* - x1-x7 contain the arguments
*
* Overview of response
* - x0 Still contains the function id
*   - This is used by TF-A to find out which function has finished
* - x1-x3 contains the return values
* - x4-x7 can contain more return values
*/
#[no_mangle]
pub extern "C" fn smc_handler(args: &mut [u64; 8])	{
	// Array with known size has same memory layout as C
	// https://doc.rust-lang.org/reference/type-layout.html
	// Arguments are in these three registers
	let svcrange = SMC_OEN_ID!(args[0]);
	//let svcrange = (args[0] >> FUNCID_OEN_SHIFT) & FUNCID_OEN_MASK;
	let func = args[0] & FUNCID_NUM_MASK;
	match svcrange {
		tsp::SVCID => {
			tsp::smc_handler(func, args);
		}
		optee::SVCID => {
			//optee::smc_handler(func, args);
			applets::smc_handler(func, args);
		}
		0x2a => {
			applets::smc_handler(func, args);
		}
		_ => {
			log::info("Invalid svc range\n");
		}
	}
}
#[no_mangle]
extern "C" fn smc_cpu_off(args: &mut [u64; 8])	{
	//let cid = cpu::id();
	//log::logf(format_args!("cpu off id: {}\n", cid));
	//for i in &mut args[1..8] { *i = 0 }
	//args[0] = tsp::OFF_DONE;
	#[cfg(spd = "tsp")]   tsp::cpu_off(args);
	#[cfg(spd = "optee")] optee::cpu_off(args);
}
#[no_mangle]
extern "C" fn smc_cpu_suspend(args: &mut [u64; 8])	{
	#[cfg(spd = "tsp")]   tsp::cpu_suspend(args);
	#[cfg(spd = "optee")] optee::cpu_suspend(args);
}
#[no_mangle]
extern "C" fn smc_cpu_resume(args: &mut [u64; 8])	{
	#[cfg(spd = "tsp")]   tsp::cpu_resume(args);
	#[cfg(spd = "optee")] optee::cpu_resume(args);
}
#[no_mangle]
extern "C" fn smc_system_off(args: &mut [u64; 8])	{
	#[cfg(spd = "tsp")]   tsp::system_off(args);
	#[cfg(spd = "optee")] optee::system_off(args);
}
#[no_mangle]
extern "C" fn smc_system_reset(args: &mut [u64; 8])	{
	#[cfg(spd = "tsp")]   tsp::system_reset(args);
	#[cfg(spd = "optee")] optee::system_reset(args);
}
#[no_mangle]
extern "C" fn smc_cpu_on(args: &mut [u64; 8])	{
	#[cfg(spd = "tsp")]   tsp::cpu_on(args);
	#[cfg(spd = "optee")] optee::cpu_on(args);
}
#[no_mangle]
extern "C" fn smc_fiq_entry(args: &mut [u64; 8])	{
	#[cfg(spd = "optee")] optee::fiq_entry(args);
}

/**
* Called when code in S-EL0 is done executing.
*
* We mimic the smc workflow used in higher layers, so S-EL0 function must
* preserve `func` parameter and send it as first argument. We then pass this
* parameter upwards according to calling specification used (like optee).
*
* When `init` is called, `func` must be set to 0, we then continue to initialize
* the next applet.
*/
pub fn smc_return(func: u64, ret: u64)	{
	if func == 0 {
		// Using a function id of 0 indicate that this is a reponse to init-function
		log::info("Back in smc_return");

		// Call next init
		applets::init();

		// If we return here, we need to return back to S-EL3
		// First unmap TTBR for S-EL-0
		mmu::switch_ttbr1(0);
		unsafe { return_el3(); }
	} else {
		#[cfg(spd = "optee")] optee::smc_return(func, ret);
	}
}
