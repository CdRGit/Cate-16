mod terminal;

mod machine;

use machine::bus::*;
use machine::cpu::*;

fn main() {
    let mut cpu = W65C816::new(Bus::new("../rom/boot_rom".to_string()));

    loop {
        if cpu.instruction() != RunStatus::Running {
            break;
        }
    }
}
