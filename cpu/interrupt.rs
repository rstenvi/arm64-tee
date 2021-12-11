use cpu::svc;
use lib::log;
/*
#[cfg(gic = "2")] type GicImpl = gic::GICv2;
//const TSP_IRQ_SEC_PHY_TIMER: u32 = 29;
const AARCH64_EXC_IRQ_SPX: u64 = 0x12;
*/

const AARCH64_EXC_SYNC_AARCH64: u64 = 0x21;
pub mod esr {
	pub const MASK: u64 = 0b111111;
	pub mod ec {
		pub const SMC: u32 = 0b010111;
		pub const SVC: u32 = 0b010101;
	}

}

#[derive(Debug, Default)]
#[repr(C)]
pub struct InterruptState {
	pub exc_type: u64,
	pub esr:      u64,
	pub saved_sp: u64,
	pub elr:      u64,
	pub spsr:     u64,
	pub regs: [u64; 31],
}

fn get_esr_ec(esr: u64) -> u32 {
	return ((esr >> 26) & esr::MASK) as u32;
}
fn handle_sync_lower(state: &mut InterruptState) {
	let ec = get_esr_ec(state.esr);
	match ec {
		esr::ec::SVC => {
			let sysno = state.regs[8];
			let args: [u64; 8] = [
				state.regs[0], state.regs[1], state.regs[2], state.regs[3],
				state.regs[4], state.regs[5], state.regs[6], state.regs[7]
			];
			state.regs[0] = svc::handle(sysno, args);
		}
		_ => {
			log::info("Unknown EC, halting...");
			loop { }
		}
	}
}

#[no_mangle]
pub extern "C" fn handle_exception(state: &mut InterruptState)	{
	match state.exc_type {
		/*AARCH64_EXC_IRQ_SPX => {
			ret = handle_irq(state);
		}*/
		AARCH64_EXC_SYNC_AARCH64 => {
			handle_sync_lower(state);
		}
		_ => {
			log::info("Unknown exception, halting...");
			loop { }
		}
	}
}
/*
fn handle_irq(_state: &mut InterruptState) -> u64	{
	let gid: u32 = GicImpl::pending();
	let ret;

	match gid {
		platform::irq::SEC_PHY_TIMER => {
			GicImpl::acknowledge();
			timer::handle_interrupt();
			GicImpl::eoi(gid);
			ret = 0;
		}
		// TODO: Probably need specific for EL1 and EL3
		_ => { ret = tsp::PREEMPTED; }
	}
	return ret;
}

*/
