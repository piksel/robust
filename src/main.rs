mod system;
mod bitflags;

use std::fs;
use anyhow::Result;

use system::cpu::CPU;

fn main() -> Result<()> {

    let mut system = system::System::new();

    let cart_file = fs::File::open("carts/nestest.nes")?;

    system.load_cart(&cart_file)?;

    println!();
    println!("Starting execution...");

    system.run()?;

    println!("Done!");

    Ok(())
}
