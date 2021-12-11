pub mod register;
pub mod smc;
pub mod interrupt;
pub mod svc;

pub fn id() -> u32 {
	// TODO: This is a quick way to get unique CPU id on qemu
	// will not necessarily work on other platforms
	let mut id = register::read::mpidr_el1();

	// Get affinity level
	id &= 0xff;

	return id as u32;
}
