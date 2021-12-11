use driver::dtb;
use lib::math;
use lib::memory;
use lib::log;
use driver::mmu;

extern "C" {
	fn memset(addr: u64, c: i8, size: u64);
}

const PHYS_PAGE_SIZE: u64 = 4096;

// TODO: Don't change this w/o changing the algorithm below
const BITS_PER_PAGE:  u64 = 1;

struct PmmData {
	/* Address to a bitmap */
	bitmap:   u64,

	/* Real start of memory */
	startmem: u64,

	/* Number of real pages we keep track of */
	pages:    u64,
}
static mut DATA: PmmData = PmmData{bitmap: 0, startmem: 0, pages: 0};

macro_rules! pages_per_byte {
	() => { (8 / BITS_PER_PAGE) };
}
macro_rules! addr_to_page {
	($addr:expr) => {
		($addr - _startmem()) / PHYS_PAGE_SIZE
	};
}
macro_rules! get_offsets {
	($page:expr) => {
		($page / (8 / BITS_PER_PAGE), $page / (8 / BITS_PER_PAGE))
	};
}
fn _bitmap() -> u64 { return unsafe { DATA.bitmap}; }
fn _startmem() -> u64 { return unsafe { DATA.startmem}; }
fn _pages() -> u64 { return unsafe { DATA.pages}; }


fn _decref(byte: u64, offs: u64)	{
	let bitmap = _bitmap();
	let mut byteval = memory::dma::read::u8(bitmap + byte);

	// TODO: Will not work if more than 1 bits per page
	byteval &= !(1 << offs);
	memory::dma::write::u8(bitmap + byte, byteval);
}

fn _addref(byte: u64, offs: u64)	{
	let bitmap = _bitmap();
	let mut byteval = memory::dma::read::u8(bitmap + byte);

	// TODO: Will not work if more than 1 bits per page
	byteval |= 1 << offs;
	memory::dma::write::u8(bitmap + byte, byteval);
}
fn find_free(bitmap: u64, pages: u64) -> i32 {
	let bytes = pages / pages_per_byte!();
	for i in 0..bytes {
		let byteval = memory::dma::read::u8(bitmap + i);
		if byteval != 0xff {
			for j in 0..8 {
				if ((1 << j) & byteval) == 0 {
					memory::dma::write::u8(bitmap + i, byteval | (1 << j));

					return ((i * 8) + j) as i32;
				}
			}
		}
	}
	return -1;
}

fn _init(startmem: u64, bitmap: u64, pages: u64) {
	unsafe {
		DATA.startmem = startmem;
		DATA.bitmap = bitmap;
		DATA.pages = pages;
	}

	// Mark entire region as free
	let bytes = pages / (8 / BITS_PER_PAGE);
	for i in 0..bytes {
		memory::dma::write::u8(bitmap + i as u64, 0);
	}
}

fn addref(addr: u64)	{
	let page = addr_to_page!(addr);
	let (byte, offs) = get_offsets!(page);
	_addref(byte, offs);
}

pub fn free(addr: u64)	{
	let page = addr_to_page!(addr);
	let (byte, offs) = get_offsets!(page);
	_decref(byte, offs);
}

pub fn mark(from: u64, to: u64)	{
	let afrom = math::align_pow2_down!(from, PHYS_PAGE_SIZE);
	let ato = math::align_pow2_up!(to, PHYS_PAGE_SIZE);
	for addr in (afrom..ato).step_by(PHYS_PAGE_SIZE as usize) {
		addref(addr);
	}
}
pub fn alloc() -> u64 {
	let idx = find_free(_bitmap(), _pages());
	if idx < 0 {
		log::info("Unable find block");
		return u64::MAX;
	}
	return _startmem() + (idx as u64 * PHYS_PAGE_SIZE);
}
pub fn allocz() -> u64 {
	let ret = alloc();
	if ret != u64::MAX {
		unsafe { memset(mmu::paddr2linear(ret), 0x00, PHYS_PAGE_SIZE); }
	}
	return ret;
}

pub fn init(imgstart: u64, imgend: u64) -> i32 {
	let (ramaddr, size) = dtb::get_secure_memory();
	let rstart = math::align_pow2_down!(imgstart, PHYS_PAGE_SIZE);
	let rend = math::align_pow2_up!(imgend, PHYS_PAGE_SIZE);

	_init(ramaddr, rend, size / PHYS_PAGE_SIZE);

	// Mark first page as taken
	addref(ramaddr);

	// Mark image region as taken
	for addr in (rstart..rend).step_by(PHYS_PAGE_SIZE as usize) {
		addref(addr);
	}

	// Mark bitmap as taken
	let bmbytes = math::align_pow2_up!(
		(size / PHYS_PAGE_SIZE) / (8 / BITS_PER_PAGE),
		PHYS_PAGE_SIZE
	);
	let bmpages = bmbytes / PHYS_PAGE_SIZE;
	let bitmap = _bitmap();
	for i in 0..bmpages {
		addref(bitmap + (i * PHYS_PAGE_SIZE) as u64);
	}
	return 0;
}
pub fn lateinit(linear: u64) -> i32 {
	unsafe {
		DATA.bitmap += linear;
	}
	return 0;
}
