
pub mod read {
	pub fn cntfrq_el0() -> u64 {
		let ret: u64;
		unsafe { asm!("mrs {ret}, cntfrq_el0", ret = out(reg) ret); }
		return ret;
	}
	pub fn cntpct_el0() -> u64 {
		let ret: u64;
		unsafe { asm!("mrs {ret}, cntpct_el0", ret = out(reg) ret); }
		return ret;
	}
	pub fn cntvct_el0() -> u64 {
		let ret: u64;
		unsafe { asm!("mrs {ret}, cntvct_el0", ret = out(reg) ret); }
		return ret;
	}
	pub fn mpidr_el1() -> u64 {
		let ret: u64;
		unsafe { asm!("mrs {ret}, mpidr_el1", ret = out(reg) ret); }
		return ret;
	}
	pub fn scr_el3() -> u64 {
		let ret: u64;
		unsafe { asm!("mrs {ret}, scr_el3", ret = out(reg) ret); }
		return ret;
	}
	pub fn tcr_el1() -> u64 {
		let ret: u64;
		unsafe { asm!("mrs {ret}, tcr_el1", ret = out(reg) ret); }
		return ret;
	}
/*	pub fn ttbr0_el1() -> u64 {
		let ret: u64;
		unsafe { asm!("mrs {ret}, ttbr0_el1", ret = out(reg) ret); }
		return ret;
	}
	pub fn ttbr1_el1() -> u64 {
		let ret: u64;
		unsafe { asm!("mrs {ret}, ttbr1_el1", ret = out(reg) ret); }
		return ret;
	}
	*/
}

pub mod write {
	pub fn cntv_cval_el1(v: u64) {
		unsafe { asm!("msr cntv_cval_el1, {v}", v = in(reg) v); }
	}
	pub fn cntps_cval_el1(v: u64) {
		unsafe { asm!("msr cntps_cval_el1, {v}", v = in(reg) v); }
	}
	pub fn cntps_ctl_el1(v: u64) {
		unsafe { asm!("msr cntps_ctl_el1, {v}", v = in(reg) v); }
	}
	pub fn scr_el3(v: u64) {
		unsafe { asm!("msr scr_el3, {v}", v = in(reg) v); }
	}
}

#[macro_export]
macro_rules! read_sctlr_el1 {
	() => {
		unsafe {
			let result: u64;
			asm!("mrs {ret}, sctlr_el1", ret = out(reg) result);
			result
		}
	};
}
#[macro_export]
macro_rules! read_ttbr0_el1 {
	() => {
		unsafe {
			let result: u64;
			asm!("mrs {ret}, ttbr0_el1", ret = out(reg) result);
			result
		}
	};
}
#[macro_export]
macro_rules! read_ttbr1_el1 {
	() => {
		unsafe {
			let result: u64;
			asm!("mrs {ret}, ttbr1_el1", ret = out(reg) result);
			result
		}
	};
}
pub(crate) use read_sctlr_el1;
pub(crate) use read_ttbr0_el1;
pub(crate) use read_ttbr1_el1;




#[macro_export]
macro_rules! write_sctlr_el1 { ($v: expr) => { unsafe { asm!("msr sctlr_el1, {v:x}", v = in(reg) $v) }; } }

#[macro_export]
macro_rules! write_tcr_el1 { ($v: expr) => { unsafe { asm!("msr tcr_el1, {v:x}", v = in(reg) $v) }; } }

#[macro_export]
macro_rules! write_ttbr0_el1 { ($v: expr) => { unsafe { asm!("msr ttbr0_el1, {v:x}", v = in(reg) $v) }; } }
#[macro_export]
macro_rules! write_ttbr1_el1 { ($v: expr) => { unsafe { asm!("msr ttbr1_el1, {v:x}", v = in(reg) $v) }; } }

pub(crate) use write_sctlr_el1;
pub(crate) use write_tcr_el1;
pub(crate) use write_ttbr0_el1;
pub(crate) use write_ttbr1_el1;

pub mod sctlr {
	pub const I:   u64 = 1 << 12;
	pub const EOS: u64 = 1 << 11;
	pub const EIS: u64 = 1 << 22;
}
