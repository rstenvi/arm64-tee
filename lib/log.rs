use driver::serial;

pub fn info(buf: &str)	{
	serial::write("I: ");
	serial::write(buf);
	serial::putc('\n');
}
pub fn debug(buf: &str) {
	serial::write("D: ");
	serial::write(buf);
	serial::putc('\n');
}
pub fn bug(buf: &str)	{
	serial::write("BUG: ");
	serial::write(buf);
	serial::putc('\n');
}
pub fn from_memory(_buf: u64) {
	let mut i = 0;
	loop {
		let buf = (_buf + i) as *const u8;
		let val = unsafe { core::ptr::read(buf) };
		if val != 0 {
			serial::putc(val as char);
		} else {
			break;
		}
		i += 1;
	}
	serial::putc('\n');
}

