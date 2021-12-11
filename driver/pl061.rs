use log;

const MAX_PL061: usize = 1;
const GPIOS_PER_DEV: u32 = 8;
const OFFSET_DIR: u64 = 0x400;

static mut BASES: [u64; MAX_PL061] = [0; MAX_PL061];

fn get_gpio_vals(gpio: u32) -> (u64, u32) {
	let idx = (gpio / GPIOS_PER_DEV) as usize;
	let offset = gpio % GPIOS_PER_DEV;
	if idx >= MAX_PL061	{ return (u64::MAX, 0) }

	unsafe { return (BASES[idx], offset); }
}

pub mod set	{
	use driver::gpio;
	use lib::memory;
	pub fn direction(gpio: u32, dir: u32)	{
		let (base, offset) = super::get_gpio_vals(gpio);
		if base == u64::MAX {
			return;
		}

		let mut data = memory::dma::read::u8(base + super::OFFSET_DIR);
	
		if dir == gpio::DIR_OUT {
			data |= 1 << offset
		}
		else	{
			data &= !(1 << offset)
		}
		memory::dma::write::u8(base + super::OFFSET_DIR, data);
	}
	pub fn value(gpio: u32, val: u8)	{
		let (base, offset) = super::get_gpio_vals(gpio);
		if base == u64::MAX {
			return;
		}
	
		let rval;
		if val == gpio::HIGH {
			rval = 1 << offset;
		}
		else {
			rval = 0;
		}
		memory::dma::write::u8(base + (1 << (offset + 2)), rval);
	}
}
pub mod get {
	use driver::gpio;
	use lib::memory;
	pub fn direction(gpio: u32) -> u32	{
		let (base, offset) = super::get_gpio_vals(gpio);
		if base == u64::MAX {
			return u32::MAX;
		}

		let val = memory::dma::read::u8(base + super::OFFSET_DIR);

		if val & (1 << offset) != 0 {
			return gpio::DIR_OUT;
		}
		return gpio::DIR_IN;
	}
	pub fn value(gpio: u32) -> u32 {
		let (base, offset) = super::get_gpio_vals(gpio);
		if base == u64::MAX {
			return u32::MAX;
		}

		// base_addr + BIT(offset + 2)
		let val = memory::dma::read::u8(base + (1 << (offset + 2)));
		if val != 0 {
			return gpio::HIGH as u32;
		}
		return gpio::LOW as u32;
	}
}

pub fn register(idx: usize, base: u64)	{
	if idx < MAX_PL061	{
		unsafe { BASES[idx] = base; }
	}
	else	{
		log::info("Tried to register PL061 controller outside index bounds\n");
	}
}
