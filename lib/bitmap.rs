use lib::memory;

extern "C" {
	fn memset(addr: u64, c: i8, size: u64);
}

fn _decref(bitmap: u64, bytes: u64, offs: u64)	{
	let byte = offs / 8;
	let bit = offs % 8;
	assert!(byte < bytes);

	let mut byteval = memory::dma::read::u8(bitmap + byte);
	byteval &= !(1 << bit);
	memory::dma::write::u8(bitmap + byte, byteval);
}

fn _addref(bitmap: u64, bytes: u64, offs: u64)	{
	let byte = offs / 8;
	let bit = offs % 8;
	assert!(byte < bytes);

	let mut byteval = memory::dma::read::u8(bitmap + byte);
	byteval |= 1 << bit;
	memory::dma::write::u8(bitmap + byte, byteval);
}
pub fn find_free(bitmap: u64, bytes: u64, num: u64) -> i32 {
	let mut matches = 0;
	for i in 0..bytes {
		let byteval = memory::dma::read::u8(bitmap + i);
		if byteval != 0xff {
			for j in 0..8 {
				if ((1 << j) & byteval) == 0 {
					matches += 1;
					if matches == num {
						return (((i * 8) + j) + 1 - matches) as i32;
					}
				} else {
					matches = 0;
				}
			}
		}
	}
	return -1;
}


pub fn free(bitmap: u64, bytes: u64, idx: u64, num: u64)	{
	for i in 0..num {
		_decref(bitmap, bytes, idx + i);
	}
}

pub fn alloc(bitmap: u64, bytes: u64, num: u64) -> u64 {
	let idx = find_free(bitmap, bytes, num);
	if idx < 0 {
		return u64::MAX;
	}
	for i in 0..num {
		_addref(bitmap, bytes, idx as u64 + i);
	}
	return idx as u64;
}
