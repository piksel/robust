use std::io::{Read, BufReader};


use self::cpu::CPU;

use self::cart::Cart;
use anyhow::Result;


pub mod cart;
pub mod bus;
pub mod cpu;

pub struct System {
    ram: Vec<u8>,
    ppu: Vec<u8>,
    apu: Vec<u8>,
    pub(crate) cpu: CPU,
    pub(crate) cart: Option<Cart>
}

impl System {
    pub fn new() -> Self {
        let cpu = CPU::init();
        
        System {
            ram: vec![0; 2048],
            ppu: vec![0; 8],
            apu: vec![0; 0x18],
            cpu,
            cart: None
        }
    }

    pub(crate) fn load_cart(&mut self, cart_file: &std::fs::File) -> Result<()> {
        let reader = BufReader::new(cart_file);
        let cart = Cart::new(reader.bytes())?;

        // for b in cart.prg_rom.iter() {
        //     print!("{b:02x} ")
        // }
        // println!();

        self.cart = Some(cart);

        println!("Cart loaded!");


        Ok(())
    }

    pub(crate) fn run(&mut self) -> Result<()> {

        // init program counter (for use with test cart)
        self.cpu.pc = 0xc000;

        for step in 0..10 {
            let pc = self.cpu.pc;
            let a = self.cpu.a;
            let x = self.cpu.x;
            let y = self.cpu.y;
            let sp = self.cpu.sp;
            print!("Step {step:02}:  PC: {pc:04x}  A: {a:02x}  X: {x:02x}  Y: {y:02x}  SP: {sp:02x}  Flags: ");

            if self.cpu.carry {
                print!("C")
            } else {
                print!("-")
            }

            if self.cpu.decimal {
                print!("D")
            } else {
                print!("-")
            }

            if self.cpu.interrupt {
                print!("I")
            } else {
                print!("-")
            }

            if self.cpu.overflow {
                print!("O")
            } else {
                print!("-")
            }

            if self.cpu.sign {
                print!("S")
            } else {
                print!("-")
            }

            if self.cpu.soft_break {
                print!("B")
            } else {
                print!("-")
            }

            if self.cpu.zero {
                print!("Z")
            } else {
                print!("-")
            }

            println!();

            let (op, am) = self.cpu.load(&self);
            self.cpu.execute(&self, op, am);
        }

        Ok(())
    }

}

