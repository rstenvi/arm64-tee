pub mod memory;
pub mod log;
pub mod alloc;
pub mod bitmap;

pub mod math {
	#[macro_export]
	macro_rules! align_pow2_up {
		($val:expr, $align:expr) => {
			// Should be relatively effective, but only works on numbers which are 1 << N
			$val + ($align - 1) & !($align - 1)
		};
	}
	pub(crate) use align_pow2_up;
	#[macro_export]
	macro_rules! align_pow2_down {
		($val:expr, $align:expr) => {
			$val & !($align - 1)
		};
	}
	pub(crate) use align_pow2_down;
}

pub mod sizes {
	pub const KB: u64 = 1024;
	pub const MB: u64 = KB * 1024;
	pub const GB: u64 = MB * 1024;
}
