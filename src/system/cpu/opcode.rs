use crate::{system::{System, addr::Addr}};

use crate::system::cpu::{self, AddressMode, Register, CPU, addr_relative, Flag, resolve_addr, get_addr_ro};

use super::{resolve_addr_with_xp, default_cycles};



#[derive(Debug)]
pub(crate) enum OpCode {
    Break,
    Kill,
    NoOp,

    // Logical operators
    // Only possible on accumulator register
    Or,
    And,
    ExOr,
    Add,
    Sub,
    Compare(Register), // CMP, CPX, CPY
    Dec(Option<Register>), // DEC works on addresses and X/Y
    Inc(Option<Register>),
    ShiftLeft,
    RotateLeft,
    ShiftRight,
    RotateRight,


    // Move
    Load(Register),
    Store(Register),
    Transfer(Register, Register),
    PushAcc,   // PHA
    PushFlags, // PHP, Push Processor
    PullAcc,   // PLA
    PullFlags, // PLP, Pull Processor

    Jump,
    JumpSub,
    BranchIf(Flag, bool),

    ReturnInt,
    ReturnSub,

    SetFlag(Flag, bool),
    Bit,

    // Hack (undocumented)
    LoadHack(Register, Register),
    StoreHack(Register, Register),
    DecCmpHack,
    IncSubHack,
    ShiftLeftOrHack,
    RotLeftAndHack,
    ShiftRightOrHack,
    RotRightAddHack,
}

impl OpCode {
    pub fn execute(&self, sys: &mut System, address_mode: &AddressMode) -> u64 {

        let cycles_start = sys.cycles;
        // sys.cpu.pc += 1i8;

        let cycles = match self {
            OpCode::Jump => {
                let addr = get_addr_ro(sys, address_mode).addr;
                sys.cpu.pc = addr; 
                match address_mode { AddressMode::Indirect(_) => 5, _ => 3 }
            }

            OpCode::JumpSub => {
                assert!(matches!(address_mode, AddressMode::Absolute(None)));
                let addr = resolve_addr(sys, address_mode);
                let pc = sys.cpu.pc - 1;
                CPU::stack_push_word(sys, pc.into());
                sys.cpu.pc = addr; 
                2
            }

            OpCode::BranchIf(flag, value) => {
                assert!(matches!(address_mode, AddressMode::Relative));
                let addr = addr_relative(sys);
                if sys.cpu.get_flag(flag) == *value {

                    let old_pc = sys.cpu.pc;
                    sys.cpu.pc += addr;
                    // If the new PC is on another page, add +2 cycles
                    if sys.cpu.pc.same_page_as(old_pc) {3} else {4}
                } else {2}
            }

            OpCode::Load(target_reg) => {
                let addr = resolve_addr(sys, address_mode);
                let value = sys.read_byte(addr);
                sys.cpu.set_reg(target_reg, value);
                sys.cpu.update_flags(value);
                0
            }

            OpCode::Store(reg) => {
                // let addr = resolve_addr(sys, address_mode);
                let addr = resolve_addr_with_xp(sys, address_mode, true);
                let value = sys.cpu.get_reg(reg);
                sys.write_byte(addr, value);
                0
            }

            OpCode::NoOp => {
                match address_mode {
                    AddressMode::Implied => {2},
                    _ => {
                        // Dummy functions still need to push the PC
                        let xp = get_addr_ro(sys, address_mode).crossed_page;
                        default_cycles(address_mode, xp)
                    }
                }
                
            }

            OpCode::SetFlag(flag, value) => {
                match flag {
                    Flag::Carry => sys.cpu.carry = *value,
                    Flag::Decimal => sys.cpu.decimal = *value,
                    Flag::Interrupt => sys.cpu.interrupt = *value,
                    Flag::Overflow => sys.cpu.overflow = *value,
                    flag => panic!("setting flag {flag:?} is not implemented")
                };
                2
            }

            OpCode::Bit => {
                let addr = resolve_addr(sys, address_mode);
                let value = sys.read_byte(addr);
                sys.cpu.zero = (sys.cpu.a & value) == 0;
                sys.cpu.overflow = (value & 0b01000000) != 0;
                sys.cpu.sign = (value & 0b10000000) != 0;
                0
            }

            OpCode::ReturnSub => {
                let pc = Addr(CPU::stack_pull_word(sys));
                sys.cpu.pc = pc + 1;
                6
            }

            OpCode::ReturnInt => {
                
                let flags = CPU::stack_pull_byte(sys);
                sys.cpu.set_status(flags);
                let pc = CPU::stack_pull_word(sys).into();
                sys.cpu.pc = pc;
                6
            }

            OpCode::PushFlags => {
                let flags = sys.cpu.status();
                // Always push the status with B bit set to the stack
                CPU::stack_push_byte(sys, flags | (1u8 << 4));
                3
            }

            
            OpCode::PullFlags => {
                let flags = CPU::stack_pull_byte(sys);
                sys.cpu.set_status(flags);
                4
            }


            OpCode::PullAcc => {
                
                let value = CPU::stack_pull_byte(sys);
                sys.cpu.set_reg(&Register::A, value);
                sys.cpu.update_flags(value);
                4
            }

            OpCode::PushAcc => {
                CPU::stack_push_byte(sys, sys.cpu.a);
                3
            }
            
            OpCode::And => {
                let addr = resolve_addr(sys, &address_mode);
                sys.cpu.a &=  sys.read_byte(addr);
                sys.cpu.update_flags(sys.cpu.a);
                0
            }

            OpCode::Add => {
                let addr = resolve_addr(sys, &address_mode);
                //let (value, carry) = sys.cpu.a.carrying_add(sys.read_byte(addr), sys.cpu.carry);
                let ack_val = sys.cpu.a;// & 0b10000000;
                let rhs_val = sys.read_byte(addr);
                
                let carry = if sys.cpu.carry {1} else {0};
                let sum = (ack_val as u16 + rhs_val as u16) + carry;

                let overflow = sum > 0xff;

                let value = sum as u8;

                let ack_sign = (ack_val & 0b10000000) != 0;
                let rhs_sign = (rhs_val & 0b10000000) != 0;
                let org_sign = ack_sign != rhs_sign;

                sys.cpu.a = value;
                sys.cpu.update_flags(value);
                sys.cpu.carry = overflow;

                sys.cpu.overflow = if overflow {false} else {org_sign != sys.cpu.sign};

                0
            }
            
            OpCode::Sub => {
                let addr = resolve_addr(sys, &address_mode);
                let sub = sys.read_byte(addr);
                let ack = sys.cpu.a;

                cpu_sub(sys, sub, ack, sys.cpu.carry);
                
                0
            }
                        
            OpCode::Or => {
                let addr = resolve_addr(sys, &address_mode);
                sys.cpu.a |=  sys.read_byte(addr);
                sys.cpu.zero = sys.cpu.a == 0;
                sys.cpu.sign = (sys.cpu.a & 0b10000000) != 0;
                0
            }

                        
            OpCode::ExOr => {
                let addr = resolve_addr(sys, &address_mode);
                sys.cpu.a ^=  sys.read_byte(addr);
                sys.cpu.zero = sys.cpu.a == 0;
                sys.cpu.sign = (sys.cpu.a & 0b10000000) != 0;
                0
            }

            OpCode::Compare(reg) => {
                let addr = resolve_addr(sys, &address_mode);
                let value_m = sys.read_byte(addr);
                let value_r = sys.cpu.get_reg(reg);
                sys.cpu.carry = value_r >= value_m;
                let val = value_r.wrapping_sub(value_m);
                sys.cpu.update_flags(val);
                sys.cpu.zero = value_r == value_m;
                //sys.cpu.soft_break = true;
                
                0
            }

            OpCode::Inc(Some(reg)) => {
                let (value, _) = sys.cpu.get_reg(reg).overflowing_add(1);
                sys.cpu.set_reg(reg, value);
                sys.cpu.update_flags(value);
                2
            }

            OpCode::Inc(None) => {
                let addr = resolve_addr_with_xp(sys, &address_mode, true);
                let (value, _) = sys.read_byte(addr).overflowing_add(1);
                sys.write_byte(addr, value);
                sys.cpu.update_flags(value);
                2
            }

            OpCode::Dec(Some(reg)) => {
                let (value, _) = sys.cpu.get_reg(reg).overflowing_sub(1);
                sys.cpu.set_reg(reg, value);
                sys.cpu.update_flags(value);
                2
            }

            OpCode::Dec(None) => {
                let addr = resolve_addr_with_xp(sys, &address_mode, true);
                let (value, _) = sys.read_byte(addr).overflowing_sub(1);
                sys.write_byte(addr, value);
                sys.cpu.update_flags(value);
                2
            }

            OpCode::Transfer(src, dst) => {
                let value = sys.cpu.get_reg(src);
                sys.cpu.set_reg(dst, value);
                match dst {
                    Register::SP => {} // No flags updated
                    _ => sys.cpu.update_flags(value)
                };
                2
            }

            OpCode::ShiftRight => {
                let (value, carry) = match address_mode {
                    AddressMode::Register(r) => {
                        let (value, carry) = shift_right(sys.cpu.get_reg(r));
                        sys.cpu.set_reg(r, value);
                        (value, carry)
                    },
                    _ => {
                        let addr = resolve_addr_with_xp(sys, &address_mode, true);
                        let (value, carry) = shift_right(sys.read_byte(addr));
                        sys.write_byte(addr, value);
                        (value, carry)
                    }
                };
                sys.cpu.carry = carry;
                sys.cpu.update_flags(value);
                2
            }

            OpCode::ShiftLeft => {
                let (value, carry) = match address_mode {
                    AddressMode::Register(r) => {
                        let (value, carry) = shift_left(sys.cpu.get_reg(r));
                        sys.cpu.set_reg(r, value);
                        (value, carry)
                    },
                    _ => {
                        let addr = resolve_addr_with_xp(sys, &address_mode, true);
                        let (value, carry) = shift_left(sys.read_byte(addr));
                        sys.write_byte(addr, value);
                        (value, carry)
                    }
                };
                sys.cpu.carry = carry;
                sys.cpu.update_flags(value);
                2
            }

            OpCode::RotateRight => {
                let (value, carry) = match address_mode {
                    AddressMode::Register(r) => {
                        let (value, carry) = rot_right(sys.cpu.get_reg(r), sys.cpu.carry);
                        sys.cpu.set_reg(r, value);
                        (value, carry)
                    },
                    _ => {
                        let addr = resolve_addr_with_xp(sys, &address_mode, true);
                        let (value, carry) = rot_right(sys.read_byte(addr), sys.cpu.carry);
                        sys.write_byte(addr, value);
                        (value, carry)
                    }
                };
                sys.cpu.carry = carry;
                sys.cpu.update_flags(value);
                2
            }

            OpCode::RotateLeft => {
                let (value, carry) = match address_mode {
                    AddressMode::Register(r) => {
                        let (value, carry) = rot_left(sys.cpu.get_reg(r), sys.cpu.carry);
                        sys.cpu.set_reg(r, value);
                        (value, carry)
                    },
                    _ => {
                        let addr = resolve_addr_with_xp(sys, &address_mode, true);
                        let (value, carry) = rot_left(sys.read_byte(addr), sys.cpu.carry);
                        sys.write_byte(addr, value);
                        (value, carry)
                    }
                };
                sys.cpu.carry = carry;
                sys.cpu.update_flags(value);
                2
            }

            OpCode::Break => {
                CPU::stack_push_word(sys, sys.cpu.pc.into());
                CPU::stack_push_byte(sys, sys.cpu.status());
                
                sys.cpu.pc = CPU::ADDR_BREAK;
                sys.cpu.soft_break = true;

                7
            }
     

            //     ██░ ██  ▄▄▄       ▄████▄   ██ ▄█▀  ██████  ▐██▌ 
            //     ▓██░ ██▒▒████▄    ▒██▀ ▀█   ██▄█▒ ▒██    ▒  ▐██▌ 
            //     ▒██▀▀██░▒██  ▀█▄  ▒▓█    ▄ ▓███▄░ ░ ▓██▄    ▐██▌ 
            //     ░▓█ ░██ ░██▄▄▄▄██ ▒▓▓▄ ▄██▒▓██ █▄   ▒   ██▒ ▓██▒ 
            //     ░▓█▒░██▓ ▓█   ▓██▒▒ ▓███▀ ░▒██▒ █▄▒██████▒▒ ▒▄▄  
            //      ▒ ░░▒░▒ ▒▒   ▓▒█░░ ░▒ ▒  ░▒ ▒▒ ▓▒▒ ▒▓▒ ▒ ░ ░▀▀▒ 
            //      ▒ ░▒░ ░  ▒   ▒▒ ░  ░  ▒   ░ ░▒ ▒░░ ░▒  ░ ░ ░  ░ 
            //      ░  ░░ ░  ░   ▒   ░        ░ ░░ ░ ░  ░  ░      ░ 
            //      ░  ░  ░      ░  ░░ ░      ░  ░         ░   ░    
            //                       ░                              
            // "Undocumented" Opcodes -------------------------------------------------        
                              
 

            OpCode::LoadHack(target_a, target_b) => {
                let addr = resolve_addr(sys, address_mode);
                let value = sys.read_byte(addr);
                sys.cpu.set_reg(target_a, value);
                sys.cpu.set_reg(target_b, value);
                sys.cpu.update_flags(value);
                0 // TODO: ??
            }

            OpCode::StoreHack(source_a, source_b) => {
                let addr = resolve_addr(sys, address_mode);
                let value_a = sys.cpu.get_reg(source_a);
                let value_b = sys.cpu.get_reg(source_b);
                let value = value_a & value_b;
                sys.write_byte(addr, value);
                0 // TODO: ??
            }

            OpCode::DecCmpHack => {
                let value_r = sys.cpu.a;

                let addr = resolve_addr(sys, &address_mode);
                let value_m = sys.read_byte(addr).wrapping_sub(1);
                sys.cpu.carry = value_r >= value_m;
                let val = value_r.wrapping_sub(value_m);
                sys.cpu.update_flags(val);
                sys.cpu.zero = value_r == value_m;
                sys.write_byte(addr, value_m);

                2
            }

            OpCode::IncSubHack => {
                // M + 1 -> M, A - M - C -> A

                let value_r = sys.cpu.a;

                let addr = resolve_addr(sys, &address_mode);
                let value_m = sys.read_byte(addr).wrapping_add(1);

                sys.write_byte(addr, value_m);

                cpu_sub(sys, value_m, value_r, sys.cpu.carry);

                2
            }

            OpCode::ShiftLeftOrHack => {
                // M = C <- [76543210] <- 0, A OR M -> A
                let addr = resolve_addr(sys, &address_mode);
                let value_m = sys.read_byte(addr);
                let (shifted, carry) = shift_left(value_m);
                sys.cpu.carry = carry;
                sys.write_byte(addr, shifted);
                sys.cpu.a |= shifted;

                2
            },

            OpCode::RotLeftAndHack => {
                // M = C <- [76543210] <- C, A AND M -> A
                let addr = resolve_addr(sys, &address_mode);
                let value_m = sys.read_byte(addr);
                let (value, carry) = rot_left(value_m, sys.cpu.carry);
                sys.cpu.carry = carry;
                sys.cpu.a &= value;
                sys.write_byte(addr, value);
                sys.cpu.update_flags(value);
                2
            }

            OpCode::ShiftRightOrHack => {
                // Lsr + Eor, M = 0 -> [76543210] -> C, A EOR M -> A
                let addr = resolve_addr(sys, &address_mode);
                let value_m = sys.read_byte(addr);
                let (shifted, carry) = shift_right(value_m);
                sys.cpu.carry = carry;
                sys.write_byte(addr, shifted);

                eprintln!("A: {:02x} {:08b}", sys.cpu.a, sys.cpu.a);
                sys.cpu.a ^= shifted;

                eprintln!("M: {value_m:02x} {value_m:08b}");
                eprintln!("S: {shifted:02x} {shifted:08b}");
                eprintln!("A: {:02x} {:08b}", sys.cpu.a, sys.cpu.a);
                2
            }

            OpCode::RotRightAddHack => {
                // Ror + Adc, M = C -> [76543210] -> C, A + M + C -> A, C
                let addr = resolve_addr(sys, &address_mode);
                let value_m = sys.read_byte(addr);
                let (rhs_val, rot_carry) = rot_right(value_m, sys.cpu.carry);
                // sys.cpu.carry = carry;
                sys.write_byte(addr, rhs_val);

                let ack_val = sys.cpu.a;
                let carry = if rot_carry {1} else {0};
                let sum = (ack_val as u16 + rhs_val as u16) + carry;

                let overflow = sum > 0xff;

                let value = sum as u8;

                let ack_sign = (ack_val & 0b10000000) != 0;
                let rhs_sign = (rhs_val & 0b10000000) != 0;
                let org_sign = ack_sign != rhs_sign;

                sys.cpu.a = value;
                sys.cpu.update_flags(value);
                sys.cpu.carry = overflow;

                sys.cpu.overflow = if overflow {false} else {org_sign != sys.cpu.sign};
                // TODO:
                2
            }

            op => panic!("opcode {op:?} is not implemented")
        };

        sys.cycles += cycles;
        sys.cycles - cycles_start
    }
}

fn cpu_sub(sys: &mut System, sub: u8, ack: u8, carry: bool) {

    let carry = if carry {0} else {1};
    let sub = sub as u16;
    let ack = ack as u16;

    // int result = A + ~Val + FC;
    let result = ack + (sub^0xff) + (carry ^ 1);

    // FV = !!((A ^ Val) & (A ^ result) & 0x80);
    sys.cpu.overflow = 0!=( (ack ^ sub) & (ack ^ result) & 0x80 );

    // FC = !(result & 0x100);
    sys.cpu.carry = 0!=(result & 0x100);
    // A = result & 0xFF;
    sys.cpu.a = result as u8;
    // FZ = (A == 0);
    sys.cpu.zero = sys.cpu.a == 0;
    // FN = (A >> 7) & 1;
    sys.cpu.sign = ((sys.cpu.a >> 7) & 1) != 0;
}



fn shift_right(v: u8) -> (u8, bool) {
    (v >> 1, (v & 0b0000_0001) != 0)
}

fn shift_left(v: u8) -> (u8, bool) {
    (v << 1, (v & 0b1000_0000) != 0)
}

fn rot_right(v: u8, carry: bool) -> (u8, bool) {
    let (value, c) = shift_right(v);
    (if carry { value | 0b1000_0000 } else { value & 0b0111_1111 }, c)
}

fn rot_left(v: u8, carry: bool) -> (u8, bool) {
    let (value, c) = shift_left(v);
    (if carry { value | 0b0000_0001 } else { value & 0b1111_1110 }, c)
}