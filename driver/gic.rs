use lib::memory;

mod gicc {
	use platform;
	use platform::Platform;
	use lib::memory;
	pub fn read(off: u64) -> u32	{
		//return memory::dma::read::u32(platform::gicc::BASE + off);
		return memory::dma::read::u32(platform::Impl::base_gicc() + off);
	}
	pub fn write(off: u64, val: u32)	{
		//memory::dma::write::u32(platform::gicc::BASE + off, val);
		memory::dma::write::u32(platform::Impl::base_gicc() + off, val);
	}
	pub mod off {
		pub const IAR1: u64 = 3 * 4;
		pub const HPPIR1: u64 = 6 * 4;
		pub const EOIR1: u64 = 4 * 4;
	}
}

pub trait GIC {
	fn acknowledge() -> u32 {
		return gicc::read(gicc::off::IAR1) & 0xffffff;
	}
	fn pending() -> u32	{
		return gicc::read(gicc::off::HPPIR1) & 0xffffff;
	}
	fn eoi(id: u32)	{
		memory::dsb::ishst!();
		gicc::write(gicc::off::EOIR1, id);
	}
}

pub struct GICv2 {
//	gicd_base: u64,
//	gicc_base: u64
}

impl GIC for GICv2 {
}
	

//pub static mut gicvar: GICv2 = GICv2{ };

/*

pub mod v3 {
	pub fn pending_interrupt() -> u32	{
		return super::common::pending();
	}
	pub fn acknowledge() -> u32	{
		return super::common::acknowledge();
	}
	pub fn eoi(id: u32)	{
		super::common::eoi(id);
	}
}*/
/*
mod common {
	use memory;
	pub fn acknowledge() -> u32 {
		return super::gicc::read(super::gicc::off::IAR1) & 0xffffff;
	}
	pub fn pending() -> u32 {
		return super::gicc::read(super::gicc::off::HPPIR1) & 0xffffff;
	}
	pub fn eoi(id: u32)	{
		memory::dsb::ishst();
		super::gicc::write(super::gicc::off::EOIR1, id);
	}
}
*/

