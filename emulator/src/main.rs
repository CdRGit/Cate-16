mod terminal;

mod machine;

use terminal::*;
use std::io::Read;
use std::io::Write;

use machine::bus::*;
use machine::cpu::*;

fn main() {
    let bus = Bus::new();
    let cpu = W65C816::new(bus);
}
