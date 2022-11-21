mod system;

use system::cpu::CPU;

fn main() {

    let cpu = CPU::init();

    println!("Hello, world!");
}
