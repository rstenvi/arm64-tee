/**
* Memory Manager Unit (MMU)
*
* Virtual memory is set up to have EL1 code running in lower half and EL0
* running in higher half. The image regions are set up with appropriate
* permissions:
*
* - .text has RX for EL1 and EL0
* - .rodata has RO for EL1
* - .data and .bss has RW for EL1
*
* A linear region is also set up, this region can be used to access all physical
* secure memory. Any code which need to access physical memory should translate
* address using `paddr2linear`. 
*
* The linear region is set up with RW permission for EL1. It is therefore
* possible to write to the .text segment using the linear region. A future
* improvement is to limit permission on the regions falling into the image.
*/
use cpu;
use lib::memory;
use lib::math;
use lib::log;
use lib::sizes;
use driver::pmm;
use driver::dtb;

extern "C" {
	fn image_fill_map(map: &ImageMap);
	fn get_pgd_el1() -> u64;
	fn memcpy(dst: u64, src: u64, size: u64);
}


const TCR_OFFSET_T0SZ:  u64 =  0;
const TCR_OFFSET_EPD0:  u64 =  7;
const TCR_OFFSET_IRGN0: u64 =  8;
const TCR_OFFSET_ORGN0: u64 = 10;
const TCR_OFFSET_SH0:   u64 = 12;
const TCR_OFFSET_TG0:   u64 = 14;
const TCR_OFFSET_T1SZ:  u64 = 16;
const TCR_OFFSET_EPD1:  u64 = 23;
const TCR_OFFSET_IRGN1: u64 = 24;
const TCR_OFFSET_ORGN1: u64 = 26;
const TCR_OFFSET_SH1:   u64 = 28;
const TCR_OFFSET_TG1:   u64 = 30;
const TCR_OFFSET_IPS:   u64 = 32;

const MMU_NORMAL_NC:       u64 = 0;
const MMU_NORMAL_WB_RA_WA: u64 = 1;
const MMU_NORMAL_WT_RA:    u64 = 2;
const MMU_NORMAL_WB_RA:    u64 = 3;

const MMU_NON_SHAREABLE:  u64 = 0;
const MMU_OUT_SHAREABLE:  u64 = 2;
const MMU_INN_SHAREABLE:  u64 = 3;

const MMU_GRANULE_4KB:  u64 = 0;
const MMU_GRANULE_64KB: u64 = 1;
const MMU_GRANULE_16KB: u64 = 2;

const MMU_IPS_32B: u64 = 0;
const MMU_IPS_36B: u64 = 1;
const MMU_IPS_40B: u64 = 2;
const MMU_IPS_42B: u64 = 3;
const MMU_IPS_44B: u64 = 4;
const MMU_IPS_48B: u64 = 5;
const MMU_IPS_52B: u64 = 6;

const MMU_ENTRY_NEXT_TBL: u64 = 1 | 2;
const MMU_ENTRY_NEXT_PAGE: u64 = 1 | 2 | (1 << 10);



const AP_RW:   u64 = 0 << 7;
const AP_RO:   u64 = 1 << 7;
const AP_EL0:  u64 = 1 << 6;
const MMU_UXN: u64 = 1 << 54;
const MMU_PXN: u64 = 1 << 53;

const MMU_NS:  u64 = 1 << 5;

/*
* Possible permissions when called from outside the MMU
*/
pub const EL0_RW: u64 = AP_RW | AP_EL0 | MMU_UXN | MMU_PXN;
pub const EL0_RO: u64 = AP_RO | AP_EL0 | MMU_UXN | MMU_PXN;
pub const EL0_RX: u64 = AP_RO | AP_EL0 | MMU_PXN;

pub const EL1_RW: u64 = AP_RW | MMU_UXN | MMU_PXN;
pub const EL1_RO: u64 = AP_RO | MMU_UXN | MMU_PXN;
pub const EL1_RX: u64 = AP_RO;

// Start linear region at index 2 in pud
pub const START_LINEAR_REGION:    u64 = 1 << 31;
pub const START_TEMP_REGION:      u64 = 1 << 34;

//pub const START_NS_LINEAR_REGION: u64 = 1 << 32;
pub const PAGE_SIZE: u64 = 4096;
pub const VA_BITS: u64 = 39;

pub fn mask_prot_el0(prot: u64) -> u64 {
	return prot & (AP_EL0 | MMU_UXN | MMU_PXN | AP_RO | AP_RW)
}
pub fn enforce_prot_el0(prot: u64) -> u64 {
	return prot | MMU_PXN | AP_EL0;
}


#[derive(Debug, Default)]
#[repr(C)]
struct MemoryRegion {
	start: u64,
	stop: u64,
}

#[derive(Debug, Default)]
#[repr(C)]
struct ImageMap {
	text: MemoryRegion,
	rodata: MemoryRegion,
	data: MemoryRegion,
}

#[derive(Debug, Default)]
struct MmuData {
	ttbr: u64,
	linear: u64,
	phys: MemoryRegion,
	image: ImageMap,
}

// All data associated with MMU
static mut MMUDATA: MmuData = MmuData{
	ttbr: 0,
	linear: 0,
	phys: MemoryRegion{start: 0, stop: 0},
	image: ImageMap{
		text: MemoryRegion{start: 0, stop: 0},
		rodata: MemoryRegion{start: 0, stop: 0},
		data: MemoryRegion{start: 0, stop: 0},
	},
};


macro_rules! mmu_idx {
	($vaddr:expr) => {
		(
			(($vaddr & (511 << 30)) >> 30),
			(($vaddr & (511 << 21)) >> 21),
			(($vaddr & (511 << 12)) >> 12)
		)
	};
}
macro_rules! mmu_oa {
	($vaddr:expr) => {
		$vaddr & (((1<<VA_BITS-12)-1)<<12)
	};
}
#[macro_export]
macro_rules! paddr_to_linear {
	($paddr:expr) => {
		unsafe { $paddr + MMUDATA.linear }
	};
}

#[macro_export]
macro_rules! inupper {
	($addr:expr) => {
		(($addr & (1 << 63)) >> 63) == 1
	};
}
pub(crate) use inupper;

#[macro_export]
macro_rules! maxmem_upper {
	() => { math::align_pow2_down!(u64::MAX, 4096) };
}
pub(crate) use maxmem_upper;

#[macro_export]
macro_rules! maxmem_lower {
	() => { math::align_pow2_down!((1 << mmu::VA_BITS)-1, mmu::PAGE_SIZE) };
}
pub(crate) use maxmem_lower;

pub const VA_RESERVED_START: u64 = sizes::GB * 8;
pub const VA_RESERVED_SIZE:  u64 = sizes::GB;
pub const VA_RESERVED_STOP:  u64 = VA_RESERVED_START + VA_RESERVED_SIZE;


// --------------------- Public API ---------------------------------- //

#[no_mangle]
pub extern "C" fn mmu_init_cpu(pud: u64) {
	let tcr = ((64-VA_BITS) << TCR_OFFSET_T0SZ) | (MMU_GRANULE_4KB << MMU_GRANULE_4KB) |
		(MMU_GRANULE_4KB << TCR_OFFSET_TG1) | ((64-VA_BITS) << TCR_OFFSET_T1SZ) |
		(MMU_IPS_32B << TCR_OFFSET_IPS);

	cpu::register::write_tcr_el1!(tcr);

	cpu::register::write_ttbr0_el1!(pud);

	// Need to synchronize instruction fetch buffer before enabling MMU
	memory::isb!();
	memory::dsb::ish!();

	// Enable MMU
	let sctlr: u64 = cpu::register::read_sctlr_el1!() | 1;
	cpu::register::write_sctlr_el1!(sctlr);
	memory::isb!();
}

pub fn init(imgstart: u64, imgend: u64) -> u64	{

	//let map = ImageMap::default();

	// Get all information about where image regions are in memory
	unsafe { image_fill_map(&MMUDATA.image) };
	let map = unsafe { &MMUDATA.image };

	// We assume 3-level page tables in the code below
	assert!(VA_BITS > 30 && VA_BITS <= 39);

	let (l1a, l2a, _l3a) = mmu_idx!(imgstart);
	let (l1b, l2b, _l3b) = mmu_idx!(imgend);

	// TODO: Should support this, but unlikely to happen
	assert!(l1a == l1b && l2a == l2b);

	// We don't have any linear memory region set up, so we follow a different
	// procedure to set up the first identity map
//	let pud = pmm::allocz();
	let pud = unsafe { get_pgd_el1() };
	let pmd = pmm::allocz();
	let ptd = pmm::allocz();

	// Write entries up until PTD
	memory::dma::write::u64(pud + (l1a * 8), pmd | MMU_ENTRY_NEXT_TBL);
	memory::dma::write::u64(pmd + (l2a * 8), ptd | MMU_ENTRY_NEXT_TBL);

	unsafe { MMUDATA.ttbr = pud; }

	for addr in (imgstart..imgend).step_by(PAGE_SIZE as usize) {
		let (_, _, l) = mmu_idx!(addr);
		memory::dma::write::u64(
			ptd + (l * 8),
			mmu_oa!(addr) | mmu_image_prot(map, addr, EL1_RO) | MMU_ENTRY_NEXT_PAGE
		);
	}
	// Ensure that table is filled before continuing
	memory::smp_mb!();

	// Create linear region
	let (ramstart, ramsize) = dtb::get_secure_memory();
	assert!(ramstart != u64::MAX && ramsize != u64::MAX);

	let ramend = ramstart + ramsize;

	// We could of course align it ourselves, but should hopefully be aligned
	assert!((ramstart % PAGE_SIZE) == 0);
	assert!((ramend % PAGE_SIZE) == 0);

	//map_region(pud, START_LINEAR_REGION + ramstart, ramstart, ramend, AP_RW | MMU_UXN | MMU_PXN);

	for addr in (ramstart..ramend).step_by(PAGE_SIZE as usize) {
		let vaddr = START_LINEAR_REGION + addr;

		// If we're in image region, we match the protection from image region,
		// otherwise it's RW
		map_page(pud, vaddr, addr, mmu_image_prot(map, addr, EL1_RW));
	}

	// Set rest of MMUDATA, image has already been set
	unsafe {
		MMUDATA.phys.start = ramstart;
		MMUDATA.phys.stop = ramend;
		MMUDATA.linear = START_LINEAR_REGION;
		MMUDATA.ttbr = pud;
	}

	mmu_init_cpu(pud);

	return unsafe { MMUDATA.linear };
}
pub fn paddr2linear(paddr: u64) -> u64 {
	return paddr_to_linear!(paddr);
}

pub fn map_dma(addrstart: u64, addrend: u64) -> i32 {
	let rstart = math::align_pow2_down!(addrstart, PAGE_SIZE);
	let rend = math::align_pow2_up!(addrend, PAGE_SIZE);
	assert!(rend > rstart);
	let pud = unsafe { MMUDATA.ttbr };

	// TODO: Should also set non-cacheable
	map_region(pud, rstart, rstart, rend, AP_RW | MMU_UXN | MMU_PXN);

	return 0;
}
pub fn alloc_pgd() -> u64 { return pmm::allocz(); }

pub fn alloc_pages_el1(vaddr: u64, pages: i32, prot: u64) -> i32 {
	let pud = cpu::register::read_ttbr0_el1!();
	return alloc_pages(pud, vaddr, pages, prot);
}
pub fn alloc_page_el1(vaddr: u64, prot: u64) -> i32 {
	let pud = cpu::register::read_ttbr0_el1!();
	return alloc_page(pud, vaddr, prot);
}
pub fn page_available_el1(vaddr: u64) -> bool {
	let pud = cpu::register::read_ttbr0_el1!();
	return pages_available(pud, vaddr, 1) == 1;
}

pub fn alloc_page(pud: u64, vaddr: u64, prot: u64) -> i32 {
	let page = pmm::allocz();
	map_page(pud, vaddr, page, prot);
	return 0;
}
pub fn switch_ttbr1(ttbr: u64) {
	cpu::register::write_ttbr1_el1!(ttbr);
}
pub fn alloc_pages(pud: u64, vaddr: u64, pages: i32, prot: u64) -> i32 {
	// Check if free first
	if pages_available(pud, vaddr, pages) == pages {
		for i in 0..pages {
			alloc_page(pud, vaddr + (i as u64 * PAGE_SIZE), prot);
		}
	} else {
		log::info("Tried to allocate pages from region already allocated");
		return i32::MIN;
	}
	return 0;
}
pub fn unmap_pages(pud: u64, vaddr: u64, pages: i32) -> i32 {
	// Unmap and free from pmm
	for i in 0..pages {
		let nvaddr = vaddr + (i as u64 * PAGE_SIZE);
		try_unmap_page(pud, nvaddr, true);
	}
	return 0;
}

fn ensure_mapped_in(pud: u64, vaddr: u64, len: u64, prot: u64) {
	let avaddr = math::align_pow2_down!(vaddr, PAGE_SIZE);
	let blocks = math::align_pow2_up!(len + (vaddr - avaddr), PAGE_SIZE);
	let pages = blocks / PAGE_SIZE;
	for i in 0..pages {
		let nvaddr = avaddr + (i * PAGE_SIZE);
		if ! try_unmap_page(pud, nvaddr, false) {
			alloc_page(pud, nvaddr, prot);
		}
	}

}
fn map_nonsecure_memory(ttbr: u64, paddr: u64, len: u64) -> (u64, u64)	{
	let rpaddr = math::align_pow2_down!(paddr, PAGE_SIZE);
	let offstart = rpaddr - paddr;
	let rlen = math::align_pow2_up!(paddr + len, PAGE_SIZE);
	let pages = rlen / PAGE_SIZE;

	// Ensure all pages are unmapped
	unmap_pages(ttbr, START_TEMP_REGION, pages as i32);

	map_region(ttbr, START_TEMP_REGION, rpaddr, rpaddr + (pages * PAGE_SIZE), EL1_RO | MMU_NS);
	return (START_TEMP_REGION + offstart, pages);

}
pub fn memcpy_ns(pud: u64, vaddr: u64, paddr: u64, len: u64, mapin: bool) -> i32	{
	if len == 0 || paddr == u64::MAX { return 0; }

	let ttbr = cpu::register::read_ttbr0_el1!();
	if mapin {
		ensure_mapped_in(pud, vaddr, len, EL0_RO);
	}
	let (npaddr, pages) = map_nonsecure_memory(ttbr, paddr, len);

	_memcpy_ns(pud, vaddr, npaddr, len);

	if mapin {
		// Unmap normal memory after we're done
		unmap_pages(ttbr, npaddr, pages as i32);
	}
	return 0;
}

fn _memcpy_ns(pud: u64, vaddr: u64, paddr: u64, len: u64)	{
	let avaddr = math::align_pow2_down!(vaddr, PAGE_SIZE);
	let off = vaddr - avaddr;
	let p1 = vaddr_to_paddr(pud, avaddr);
	let p2 = vaddr_to_paddr(pud, math::align_pow2_down!(vaddr + len, PAGE_SIZE));

	let lin = paddr_to_linear!(p1 + off);
	if p1 == p2 {
		unsafe { memcpy(lin, paddr, len) };
	} else {
		let copy = math::align_pow2_up!(vaddr, PAGE_SIZE) - vaddr;
		unsafe { memcpy(lin, paddr, copy) };
		return _memcpy_ns(pud, vaddr + copy, paddr + copy, len - copy);
	}
}


// ---------------------------- Internal functions ------------------------ //

fn get_tbl(tbl: u64, idx: u64, create: bool) -> u64 {
	let e1 = memory::dma::read::u64(paddr_to_linear!(tbl + (idx * 8)));
	if e1 != 0 {
		return mmu_oa!(e1);
	} else if create {
		let t1 = pmm::allocz();
		memory::dma::write::u64(paddr_to_linear!(tbl + (idx * 8)), t1 | MMU_ENTRY_NEXT_TBL);
		memory::smp_mb!();

		return t1;
	}
	return 0;
}
fn map_page(pud: u64, vaddr: u64, paddr: u64, prot: u64) {
	let (l1, l2, l3) = mmu_idx!(vaddr);
	let t2 = get_tbl(pud, l1, true);
	let t3 = get_tbl(t2, l2, true);
	let ridx = paddr_to_linear!(t3 + (l3 * 8));
	let entry = memory::dma::read::u64(ridx);
	if entry != 0 {
		pmm::free( mmu_oa!(entry) );
	}
	memory::dma::write::u64(ridx, mmu_oa!(paddr) | prot | MMU_ENTRY_NEXT_PAGE);
	memory::smp_mb!();
}
fn map_region(pud: u64, startvaddr: u64, start: u64, stop: u64, prot: u64)	{
	for addr in (start..stop).step_by(PAGE_SIZE as usize) {
		let vaddr = startvaddr + (addr - start);
		map_page(pud, vaddr, addr, prot);
	}
}
fn vaddr_to_paddr(pud: u64, vaddr: u64) -> u64	{
	let (l1, l2, l3) = mmu_idx!(vaddr);
	let t2 = get_tbl(pud, l1, false);
	if t2 == 0 { return u64::MAX; }

	let t3 = get_tbl(t2, l2, false);
	if t3 == 0 { return u64::MAX; }

	let ridx = paddr_to_linear!(t3 + (l3 * 8));
	let entry = memory::dma::read::u64(ridx);
	let ret = mmu_oa!(entry);
	if ret == 0 { return u64::MAX; }

	return ret;
}

/**
* Return true if page is mapped and false if it isn't.
*
* If dounmap is true, it will unmap the page as well.
*/
fn try_unmap_page(pud: u64, vaddr: u64, dounmap: bool) -> bool {
	let (l1, l2, l3) = mmu_idx!(vaddr);
	let t2 = get_tbl(pud, l1, false);
	if t2 == 0 { return false; }

	let t3 = get_tbl(t2, l2, false);
	if t3 == 0 { return false; }

	let ridx = paddr_to_linear!(t3 + (l3 * 8));
	let entry = memory::dma::read::u64(ridx);
	if entry != 0 {
		if dounmap {
			pmm::free( mmu_oa!(entry) );
		}
		return true
	}

	return false;
}
fn mmu_image_prot(map: &ImageMap, addr: u64, default: u64) -> u64 {
	if addr >= map.text.start && addr <= map.text.stop {
		// We share code segment with user-mode so this must be accessible to to EL0 as well
		return AP_RO | AP_EL0;
	} else if addr >= map.rodata.start && addr <= map.rodata.stop {
		// TODO:
		// Ideally, this region should only be accessible to EL1, but there is
		// some global data accessed when allocating dynamic memory. A better
		// solution would be to isolate those variables, place them in a
		// separate page and have only those variables accessible in EL0.
		return AP_RO | MMU_UXN | MMU_PXN | AP_EL0;
		//return AP_RO | MMU_UXN | MMU_PXN;
	} else if addr >= map.data.start && addr <= map.data.stop {
		return AP_RW | MMU_UXN | MMU_PXN;
	} else {
		return default;
	}
}

fn pages_available(pud: u64, vaddr: u64, pages: i32) -> i32 {
	for i in 0..pages {
		if try_unmap_page(pud, vaddr + (i as u64 * PAGE_SIZE), false) {
			return i;
		}
	}
	return pages;
}
