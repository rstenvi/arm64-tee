
#[macro_export]
macro_rules! isb {
	() => { unsafe { asm!("isb"); } };
}

#[macro_export]
macro_rules! dmb {
	() => { unsafe { asm!("dmb sy"); } };
}
#[macro_export]
macro_rules! dcinval {
	($addr:expr) => {
		memory::dmb!();
		unsafe {
			asm!("dc zva, {v:x}", v = in(reg) $addr);
		}
		memory::dsb::sy!()
	};
}

pub(crate) use isb;
pub(crate) use dmb;


pub mod dsb {
	#[macro_export]
	macro_rules! ishst {
		() => { unsafe { asm!("dsb ishst"); } };
	}
	#[macro_export]
	macro_rules! ish {
		() => { unsafe { asm!("dsb ish"); } };
	}
	#[macro_export]
	macro_rules! sy {
		() => { unsafe { asm!("dsb sy"); } };
	}
	pub(crate) use ishst;
	pub(crate) use ish;
	pub(crate) use sy;
}


#[macro_export]
macro_rules! smp_mb {
	() => {
		memory::dmb!();
		memory::dsb::sy!();
	};
}
pub(crate) use smp_mb;

pub mod dma {
	pub mod write {
		pub fn u8(addr: u64, v: u8)	{
			unsafe { core::ptr::write_volatile(addr as *mut u8, v); }
		}
		pub fn u16(addr: u64, v: u16)	{
			unsafe { core::ptr::write_volatile(addr as *mut u16, v); }
		}
		pub fn u32(addr: u64, v: u32)	{
			unsafe { core::ptr::write_volatile(addr as *mut u32, v); }
		}
		pub fn u64(addr: u64, v: u64)	{
			unsafe { core::ptr::write_volatile(addr as *mut u64, v); }
		}
	}
	pub mod read {
		pub fn u8(addr: u64) -> u8	{
			unsafe { return core::ptr::read_volatile(addr as *mut u8); }
		}
		pub fn u16(addr: u64) -> u16	{
			unsafe { return core::ptr::read_volatile(addr as *mut u16); }
		}
		pub fn u32(addr: u64) -> u32	{
			unsafe { return core::ptr::read_volatile(addr as *mut u32); }
		}
		pub fn u64(addr: u64) -> u64	{
			unsafe { return core::ptr::read_volatile(addr as *mut u64); }
		}
	}
}

pub mod be {
	pub mod read {
		macro_rules! be2host {
			($num:expr) => {
				if cfg!(target_endian = "little") {
					return $num.swap_bytes();
				} else {
					return $num;
				}
			};
		}
		pub fn u32(addr: u64) -> u32 {
			let ret = unsafe { core::ptr::read(addr as *const u32) };
			be2host!(ret)
		}
	}
	pub mod swap {
		macro_rules! swapbe2host {
			($addr:expr) => {
				if cfg!(target_endian = "little") {
					let ret = unsafe { core::ptr::read($addr) };
					unsafe { core::ptr::write($addr, ret.swap_bytes()); }
				}
			};
		}
		pub fn u16(addr: u64) { swapbe2host!(addr as *mut u16) }
		pub fn u32(addr: u64) { swapbe2host!(addr as *mut u32) }
		pub fn u64(addr: u64) { swapbe2host!(addr as *mut u64) }

	}
}

pub mod cstring {
	pub fn strlen(addr: *const u8) -> usize {
		let mut p = addr;
		while unsafe { *p } != b'\0' {
        	p = unsafe { p.add(1) };
    	}
		return (p as usize) - (addr as usize);
	}
	pub fn bytesequals(addr: *const u8, cmp: &[u8], start: usize) -> usize {
		let mut p = addr;
		let mut count = start;
		let maxlen = cmp.len();
		while unsafe { *p } != b'\0' && count < maxlen {
			if unsafe { *p } != cmp[count] {
				break;
			}
			p = unsafe { p.add(1) };
			count += 1
		}
		return (count - start) as usize;

	}
	pub fn strequals(addr: *const u8, s: &str) -> usize {
		let cmp = s.as_bytes();
		return bytesequals(addr, cmp, 0);
	}
}
