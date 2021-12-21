use lib::memory;
use lib::log;
use lib::math;

const FDT_MAGIC: u32 = 0xd00dfeed;

const FDT_BEGIN_NODE: u32 = 0x00000001;
const FDT_END_NODE: u32 = 0x00000002;
const FDT_PROP: u32 = 0x00000003;
const FDT_NOP: u32 = 0x00000004;
const FDT_END: u32 = 0x00000009;

const SIZE_CELL_DEFAULT: u32 = 1;
const ADDR_CELL_DEFAULT: u32 = 2;

static mut FDT: u64 = 0;

// #[repr(C)]
// pub struct fdtHeader {
// 	magic: u32,
// 	totalsize: u32,
//     off_dt_struct: u32,
//     off_dt_strings: u32,
//     off_mem_rsvmap: u32,
//     version: u32,
//     last_comp_version: u32,
//     boot_cpuid_phys: u32,
//     size_dt_strings: u32,
//     size_dt_struct: u32,
// }

#[derive(Debug, Default, Clone, Copy)]
pub struct FdtProp {
	size_cells: u32,
	addr_cells: u32,
	addr: u64,
	size: usize,
	status: u64
}


/**
* Find a property in Flattened Device Tree (FDT).
*
* The function returns start of property data and the length of the property
* data. If no property with the name was found, (0, 0) is returned.
*/
fn _find_prop(header: u64, _name: &str, result: &mut FdtProp, iskip: i32) -> i32	{
	let asint = header as u64;
	let mut skip = iskip;

	let off_dt_struct = memory::be::read::u32(header + 8);
	let off_dt_strings = memory::be::read::u32(header + 12);

	let name = _name.as_bytes();

	let strdata = asint + off_dt_strings as u64;
	let mut curr = asint + off_dt_struct as u64;
	let mut node: u32 = 0;
	let mut matchcount = 0;
	let mut innode: bool = false;
	let mut nodefinished: bool = false;

	result.size_cells = SIZE_CELL_DEFAULT;
	result.addr_cells = ADDR_CELL_DEFAULT;
	result.status = 0;

	let lensize = "#size-cells".len();
	let addrsize = "#address-cells".len();

	while node != FDT_END {
		node = memory::be::read::u32(curr);
		curr += 4;
		match node {
			FDT_BEGIN_NODE => {
				// Node name follows directly after identifier
// 				log::info("FDT begin node");
// 				log::from_memory(curr);
				let nodelen = memory::cstring::strlen(curr as *const u8);
				let mcount = memory::cstring::bytesequals(curr as *const u8, name, matchcount as usize);

				// Given string should always be separated by '/'
// 				let hitb = name[matchcount+mcount];
				if matchcount+mcount >= name.len() || name[matchcount+mcount] == b'/' {
					// Whole property is matched
					if mcount == nodelen {
						innode = true;
						matchcount += mcount + 1;
					}
					// We've matched everything up until '@'
					// Data after '@' is often address, we don't know that at compile time
					else if unsafe { *((curr + mcount as u64) as *const u8) } == b'@' {
						// We only skip on entries using '@' because those are
						// the only ones where we might have duplicates
						if skip == 0 {
							innode = true;
							matchcount += mcount + 1;
							if matchcount >= name.len() {
								nodefinished = true;
							}
						} else {
							skip -= 1;
							innode = false;
						}
					}
					else {
						innode = false;
					}
				} else {
					innode = false;
				}

				// Strings always have a NULL-byte, so length is strlen + 1 and aligned up to 4
				curr += math::align_pow2_up!( nodelen as u64 + 1, 4);
			}
			FDT_END_NODE => {
// 				log::info("FDT end node");
				innode = false;
				if nodefinished {
					return 0;
				}
			}
			FDT_PROP => {
				let proplen = memory::be::read::u32(curr);
				let stroffset = memory::be::read::u32(curr + 4);
				let propname = (strdata + stroffset as u64) as *const u8;
// 				log::from_memory(propname as u64);
				if innode {
					let propnamelen = memory::cstring::strlen(propname);
					if memory::cstring::bytesequals(propname, name, matchcount as usize) == propnamelen {
						if propnamelen + matchcount >= name.len() {
							result.addr = (curr + 8) as u64;
							result.size = proplen as usize;

							// We can't return quite yet, because we might still
							// need to read size-cells and address-cells. We
							// instead mark that we should return when the node
							// end
							nodefinished = true;
						}
					}
					
					// Need to keep track of size-cells and address-cells. The
					// last of these properties before the node we're looking
					// for are the correct ones.
					if proplen == 4 {
						if memory::cstring::strequals(propname, "#size-cells") == lensize {
							result.size_cells = memory::be::read::u32(curr + 8);
						}
						if memory::cstring::strequals(propname, "#address-cells") == addrsize {
							result.size_cells = memory::be::read::u32(curr + 8);
						}
					} else if memory::cstring::strequals(propname, "status") == "status".len() {
						result.status = curr + 8;
					}
				}
				// Length specifier + str offset + property data
				curr += 4 as u64 + 4 as u64 + math::align_pow2_up!(proplen, 4) as u64;
			}
			FDT_END => {
// 				log::info("Found end of FDT structs");
			}
			FDT_NOP => {
				// This does nothing and have no data
// 				log::info("Found end of FDT NOP");
			}
			_ => {
// 				log::info("Unknown FDT node");
			}
		}
	}
	// No match was found
	return -1;
}

pub fn find_prop(name: &str, result: &mut FdtProp, skip: i32) -> i32 {
	return unsafe { _find_prop(FDT, name, result, skip) };
}
pub fn get_compatible() {
	let mut prop = FdtProp::default();
	find_prop("/compatible", &mut prop, 0);
}
fn interpret_as_reg(prop: &FdtProp) -> (u64, u64) {
	// Sanity check for bugs in our code or bad data from FDT
	if prop.size != ((prop.addr_cells * 4) + (prop.size_cells * 4)) as usize {
		log::info("Not enough space reserves for reg");
		return (u64::MAX, u64::MAX);
	}

	let mut curr = prop.addr;
	let mut addr = memory::be::read::u32(curr) as u64;
	curr += 4;
	if prop.addr_cells == 2 {
		addr <<= 32;
		addr += memory::be::read::u32(curr) as u64;
		curr += 4;
	}
	let mut size = memory::be::read::u32(curr) as u64;
	curr += 4;
	if prop.size_cells == 2 {
		size <<= 32;
		size += memory::be::read::u32(curr) as u64;
	}
	return (addr, size);
}
pub fn get_as_reg(name: &str, skip: i32) -> (u64, u64) {
	let mut prop = FdtProp::default();
	if find_prop(name, &mut prop, skip) == 0 {
		if prop.status != 0 {
			let statname = prop.status as *const u8;
			if memory::cstring::strequals(statname, "disabled") == "disabled".len() {
				return get_as_reg(name, skip+1);
			}
		}
		return interpret_as_reg(&prop);
	}
	return (u64::MAX, u64::MAX);
}

/*
/secram@e000000
 secure-status
 status
 reg
 device_type
*/
pub fn get_secure_memory() -> (u64, u64) {
	let mut prop = FdtProp::default();
	if find_prop("/secram/reg", &mut prop, 0) == 0 {
		return interpret_as_reg(&prop);
	}
	return (u64::MAX, u64::MAX);
}

pub fn init(start: u64) -> u32	{
	log::info("Initializing FDT");

	// Basic sanity check so that we don't read arbitrary values
	let magic = memory::be::read::u32(start);
	if magic != FDT_MAGIC	{
		log::info("Incorrect FDT header");
		return u32::MAX;
	}

	// Store header in global variable
	unsafe { FDT = start; }

	// We got our own secure-chosen environment we could parse
	/*
	/secure-chosen
	 stdout-path
	 kaslr-seed
	*/
	let ret = memory::be::read::u32(unsafe{FDT} + 4);
	return ret;
}

