/*
Registers	Channel	    Units
$4000-$4003	Pulse 1	    Timer, length counter, envelope, sweep
$4004-$4007	Pulse 2	    Timer, length counter, envelope, sweep
$4008-$400B	Triangle	Timer, length counter, linear counter
$400C-$400F	Noise	    Timer, length counter, envelope, linear feedback shift register
$4010-$4013	DMC	        Timer, memory reader, sample buffer, output unit
$4015	    All	        Channel enable and length counter status
$4017	    All	        Frame counter
*/

use super::System;

pub struct APU {
    pub mem: Vec<u8>,
    pub polling_controller: bool,
    pub polling_expansion: bool,
    pub controller1: ControllerState,
    pub controller2: ControllerState,
}

pub struct ControllerState {
    buttons: u8,
    step: u8,
}

pub enum ControllerButton {
    Right = 1 << 7,
    Left = 1 << 6,
    Down = 1 << 5,
    Up = 1 << 4,
    Start = 1 << 3,
    Select = 1 << 2,
    B = 1 << 1,
    A = 1 << 0,
}

impl ControllerState {

    pub fn right(&self)  -> bool { self.get(7) }
    pub fn left(&self)   -> bool { self.get(6) }
    pub fn down(&self)   -> bool { self.get(5) }
    pub fn up(&self)     -> bool { self.get(4) }
    pub fn start(&self)  -> bool { self.get(3) }
    pub fn select(&self) -> bool { self.get(2) }
    pub fn b(&self)      -> bool { self.get(1) }
    pub fn a(&self)      -> bool { self.get(0) }

    pub fn set_right(&mut self, v: bool)  { self.set(7, v) }
    pub fn set_left(&mut self, v: bool)   { self.set(6, v) }
    pub fn set_down(&mut self, v: bool)   { self.set(5, v) }
    pub fn set_up(&mut self, v: bool)     { self.set(4, v) }
    pub fn set_start(&mut self, v: bool)  { self.set(3, v) }
    pub fn set_select(&mut self, v: bool) { self.set(2, v) }
    pub fn set_b(&mut self, v: bool)      { self.set(1, v) }
    pub fn set_a(&mut self, v: bool)      { self.set(0, v) }
    
    pub fn poll(&mut self) -> bool {
        let value = self.get(self.step);
        self.step = if self.step == 7 {0} else {self.step + 1};
        value
    }

    pub fn set_button(&mut self, button: ControllerButton, pressed: bool) {
        if pressed {
            self.buttons |= button as u8
        } else {
            self.buttons &= 0xff ^ button as u8
        }
    }
    

    fn set(&mut self, index: u8, value: bool) {
        if value {
            self.buttons |= mask(index)
        } else {
            self.buttons &= 0xff ^ mask(index)
        }
    }

    pub fn get(&self, index: u8) -> bool {
        self.buttons & mask(index) != 0
    }
}

fn mask(index: u8) -> u8 {1 << index}

impl APU {
    pub fn init() -> Self {
        Self {
            mem: vec![0; 0x18],
            polling_controller: false,
            polling_expansion: false,
            controller1: ControllerState{buttons: 0, step: 0},
            controller2: ControllerState{buttons: 0, step: 0},
        }
    }

    pub fn set_controller_button(&mut self, controller: usize, button: ControllerButton, pressed: bool) {
        match controller {
            0 => self.controller1.set_button(button, pressed),
            1 => self.controller2.set_button(button, pressed),
            cid => panic!("invalid controller: {cid}")
        }
    }
}

pub fn read(sys: &mut System, addr: u8) -> u8 {
    // eprintln!("Read from APU ${:02x}", addr);
    match addr {
        0x16 => if sys.apu.controller1.poll() {0} else {1},
        0x17 => if sys.apu.controller2.poll() {0} else {1},

        _ => sys.apu.mem[addr as usize],
    }
}

pub(crate) fn write(sys: &mut System, addr: u8, value: u8) {
    match addr {
        0x16 => {
            //eprintln!("POLLING CONTROLLER! {:08b}", value);
            sys.apu.polling_controller = (value & 1) != 0;
            sys.apu.polling_expansion = (value & 2) != 0;
        }
        _ => sys.apu.mem[addr as usize] = value
    }
}