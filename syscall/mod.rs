use lib::math;
use lib::log;
use driver::mmu;
use cpu;


pub fn mmap(addr: u64, size: u64, prot: u64) -> u64 {
	// Align down both address and size
	let rsize = math::align_pow2_up!(size, mmu::PAGE_SIZE);
	let raddr = math::align_pow2_down!(addr, mmu::PAGE_SIZE);
	
	// Check that addr range belongs to user mode
	// We do an assert here because it's a bug in our code if it fails
	assert!(mmu::inupper!(raddr));
	assert!(mmu::inupper!(raddr + rsize));

	// Ensure address is not in reserved range
	assert!(raddr < mmu::VA_RESERVED_START || raddr > mmu::VA_RESERVED_STOP);
	assert!(raddr + rsize < mmu::VA_RESERVED_START || raddr + rsize > mmu::VA_RESERVED_STOP);

	let mut rprot = mmu::mask_prot_el0(prot);
	if rprot != prot {
		// Bug in our code, 
		log::bug("Had to mask out protection value from EL0");
	}

	rprot = mmu::enforce_prot_el0(rprot);
	let ttbr = cpu::register::read_ttbr1_el1!();
	let res = mmu::alloc_pages(ttbr, raddr, (rsize / mmu::PAGE_SIZE) as i32, rprot);
	if res != 0 {
		return u64::MAX;
	}

	return raddr;
}

pub fn munmap(addr: u64, size: u64) -> u64 {
	// We can still get wrong size from user-mode, because we may have increased
	// it in mmap and we do not return the real size to user
	let rsize = math::align_pow2_up!(size, mmu::PAGE_SIZE);

	// Addr is returned and user-mode should use the correct addr in unmap
	assert!(math::align_pow2_down!(addr, mmu::PAGE_SIZE) == addr);


	let ttbr = cpu::register::read_ttbr1_el1!();
	mmu::unmap_pages(ttbr, addr, (rsize / mmu::PAGE_SIZE) as i32);
	return 0;
}
