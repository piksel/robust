mod system;
mod bitflags;
mod util;

use std::fs;
use anyhow::Result;
use std::env;

fn main() -> Result<()> {

    color_backtrace::install();
    let mut system = system::System::new();

    let cart_file = fs::File::open("carts/nestest.nes")?;
    let log_file = fs::File::open("carts/nestest.log")?;

    system.load_cart(&cart_file)?;

    eprintln!();
    eprintln!("Starting execution...");

    let mut args = env::args();
    let a1 = args.next();
    eprintln!("");

    let steps: u32 = args.next().map(|s| s.parse().unwrap())
        .unwrap_or(2048u32);

    system.load_expected_log(&log_file)?;
    system.run(steps).or_else(|e| {
        eprintln!("\nStack:");
        system.print_stack()?;
        eprintln!();
        Err(e)
    })?;

    eprintln!("Done!");

    Ok(())
}
