use driver;
use driver::gpio;
use driver::mmu;


pub mod gicr {
	pub const BASE: u64 = 0x80A0000;
}
pub mod uart {
	pub const BASE0: u64 = 0x09000000;
}

pub mod rom {
	pub const BASE: u64 = 0x0;
	pub const SIZE: u64 = 0x00020000;
}

pub mod irq {
	pub const SEC_PHY_TIMER: u32 = 29;
}

/* BL32 can be loaded in dram or sram */

pub mod sec {
	pub mod sram {
		/* Start of BL */
		pub const BASE: u64 = 0x0e000000;

		// Can use memory from image address and to end
		pub const SIZE: u64 = 0x00060000;


		// Unsure about these values
		const SHARED: u64 = 0x00001000;
		pub const BL32_START: u64 = BASE + SHARED;
	}

	pub mod dram {
		pub const BASE: u64 = 0x0e100000;
		pub const SIZE: u64 = 0x00f00000;
		pub const LIMIT: u64 = BASE + SIZE;
	}
}


/*
* Main interface to be used by other code
*/
pub mod memory {
	pub const BASE: u64 = super::sec::dram::BASE;
	pub const SIZE: u64 = super::sec::dram::SIZE;
	pub const LIMIT: u64 = super::sec::dram::LIMIT;
}

pub trait Platform {
	fn power_off()		{ }
	fn power_reset()	{ }
	fn base_gicc() -> u64 { return u64::MAX; }
	fn base_gicd() -> u64 { return u64::MAX; }
	fn map_dma() { }
}

pub struct PlatQemu;

impl Platform for PlatQemu {
	fn power_off()	{
		driver::pl061::set::direction(gpio::qemu::POWEROFF, gpio::DIR_OUT);
		driver::pl061::set::value(gpio::qemu::POWEROFF, gpio::HIGH);
		driver::pl061::set::value(gpio::qemu::POWEROFF, gpio::LOW);
	}
	fn power_reset()	{
		driver::pl061::set::direction(gpio::qemu::RESET, gpio::DIR_OUT);
		driver::pl061::set::value(gpio::qemu::RESET, gpio::HIGH);
		driver::pl061::set::value(gpio::qemu::RESET, gpio::LOW);
	}
	fn base_gicc() -> u64 { return 0x8010000; }
	fn base_gicd() -> u64 { return 0x8000000; }
	fn map_dma() {
		mmu::map_dma(uart::BASE0, uart::BASE0 + 4096);
	}
}



#[cfg(platform = "qemu")] pub type Impl = PlatQemu;

