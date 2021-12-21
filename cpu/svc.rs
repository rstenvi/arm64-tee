use syscall;
use applets;
use cpu::smc;

pub const SYSNO_EXIT:   u64 = 1;
pub const SYSNO_MMAP:   u64 = 2;
pub const SYSNO_MUNMAP: u64 = 3;
pub const SYSNO_STORE_PTR: u64 = 4;

pub fn handle(sysno: u64, args: [u64; 8]) -> u64 {
	let ret: u64;
	match sysno {
		SYSNO_EXIT => {
			ret = 0;
			// Need to perform smc call back to S-EL3
			smc::smc_return(args[0], args[1]);

			// Will never reach here
		}
		SYSNO_MMAP => {
			ret = syscall::mmap(args[0], args[1], args[2]);
		}
		SYSNO_MUNMAP => {
			ret = syscall::munmap(args[0], args[1]);
		}
		SYSNO_STORE_PTR => {
			ret = applets::store_ptr(args[0]);
		}
		_ => { return u64::MAX; }
	}
	return ret;
}
