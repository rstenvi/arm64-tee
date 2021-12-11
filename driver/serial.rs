use lib::memory;
use platform;

pub fn putc(c: char) {
	memory::dma::write::u32(platform::uart::BASE0, c as u32);
}

pub fn write(buf: &str) {
	for c in buf.chars() { putc(c); }
}
pub fn rx_waiting() -> bool {
	let val = memory::dma::read::u32(platform::uart::BASE0);
	return (val & (1 << 4)) == 0;
}
