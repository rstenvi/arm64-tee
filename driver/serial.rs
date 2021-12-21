use lib::memory;
use platform;

static mut UARTBASE: u64 = platform::uart::BASE0;

pub fn putc(c: char) {
	memory::dma::write::u32(unsafe {UARTBASE}, c as u32);
}
pub fn write(buf: &str) {
	for c in buf.chars() { putc(c); }
}
pub fn rx_waiting() -> bool {
	let val = memory::dma::read::u32(unsafe {UARTBASE});
	return (val & (1 << 4)) == 0;
}
pub fn set_base(base: u64) {
	unsafe { UARTBASE = base; }
}
