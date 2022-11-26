use crate::{system::System, util::carrying_add};

use super::{AddressMode, Register, get_addr, CPU, addr_relative, addr_absolute, addr_zero, Flag};



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
    ShiftLeft,  // TODO: ASL  {adr}:={adr}*2
    RotateLeft, // TODO: ROL  {adr}:={adr}*2+C
    ShiftRight, // TODO: LSR  {adr}:={adr}/2
    RotateRight, // TODO: ROR  {adr}:={adr}/2+C*128


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
    DecHack
}

impl OpCode {
    pub fn Execute(&self, sys: &mut System, address_mode: &AddressMode) -> u8 {

        let mut pc_set = false;

        let cycles = match self {
            OpCode::Jump => {
                let addr = get_addr(sys, address_mode);
                sys.cpu.pc = addr; 
                pc_set = true;
                match address_mode { AddressMode::Indirect(_) => 5, _ => 3 }
            }

            OpCode::JumpSub => {
                assert!(matches!(address_mode, AddressMode::Absolute(None)));
                let addr = addr_absolute(sys, &None);
                let pc = sys.cpu.pc;
                CPU::stack_push_word(sys, pc);
                sys.cpu.pc = addr; 
                pc_set = true;
                3
            }

            OpCode::BranchIf(flag, value) => {
                assert!(matches!(address_mode, AddressMode::Relative));
                let addr = addr_relative(sys);
                if sys.cpu.get_flag(flag) == *value {
                    sys.cpu.pc = sys.cpu.pc.checked_add_signed(addr as i16).expect("program counter overflow!");
                    // pc_set = true;
                    3
                } else {2}
            }

            OpCode::Load(target_reg) => {
                let addr = get_addr(sys, address_mode);
                let value = sys.read_byte(addr);
                sys.cpu.set_reg(target_reg, value);
                sys.cpu.update_flags(value);
                6 // TODO: should be 5 for Y if no page is crossed
            }

            OpCode::LoadHack(target_a, target_b) => {
                let addr = get_addr(sys, address_mode);
                let value = sys.read_byte(addr);
                sys.cpu.set_reg(target_a, value);
                sys.cpu.set_reg(target_b, value);
                sys.cpu.update_flags(value);
                6 // TODO: ??
            }

            OpCode::StoreHack(source_a, source_b) => {
                let addr = get_addr(sys, address_mode);
                let value_a = sys.cpu.get_reg(source_a);
                let value_b = sys.cpu.get_reg(source_b);
                let value = value_a & value_b;
                sys.write_byte(addr, value);
                6 // TODO: ??
            }

            OpCode::DecHack => {
                let (value_r, _) = sys.cpu.a.overflowing_sub(1);
                // sys.cpu.a = value_r;
                // sys.cpu.update_flags(value);

                let addr = get_addr(sys, &address_mode);
                let value_m = sys.read_byte(addr);
                // let value_r = sys.cpu.get_reg(reg);
                sys.cpu.carry = value_r >= value_m;
                let val = value_r.wrapping_sub(value_m);
                sys.cpu.update_flags(val);
                sys.cpu.zero = value_r == value_m;
                0 // TODO: ??
            }

            OpCode::Store(reg) => {
                let addr = get_addr(sys, address_mode);
                let value = sys.cpu.get_reg(reg);
                sys.write_byte(addr, value);
                match (reg, address_mode) { 
                    (Register::A, AddressMode::Zero(None)) => 3, 
                    (Register::A, AddressMode::Zero(Some(Register::X))) => 4, 
                    (Register::A, AddressMode::Absolute(None)) => 4, 
                    (Register::A, AddressMode::Absolute(Some(Register::X))) => 5, 
                    (Register::A, AddressMode::Absolute(Some(Register::Y))) => 5, 
                    (Register::A, AddressMode::Indirect(Some(Register::X))) => 6, 
                    (Register::A, AddressMode::Indirect(Some(Register::Y))) => 6, 
                    (_, AddressMode::Zero(None)) => 3, 
                    (_, AddressMode::Zero(Some(Register::Y))) => 4, 
                    (_, AddressMode::Zero(Some(Register::X))) => 4, 
                    (_, AddressMode::Absolute(None)) => 4, 
                    _ => panic!("invalid address mode {address_mode:?} for store")
                }
                
            }

            OpCode::NoOp => {
                match address_mode {
                    AddressMode::Implied => {},
                    _ => {
                        // Dummy functions still need to push the PC
                        let _ = get_addr(sys, address_mode);
                    }
                }
                2
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
                let addr = get_addr(sys, address_mode);
                let value = sys.read_byte(addr);
                sys.cpu.zero = (sys.cpu.a & value) == 0;
                sys.cpu.overflow = (value & 0b01000000) != 0;
                sys.cpu.sign = (value & 0b10000000) != 0;
                match address_mode { AddressMode::Zero(_) => 3, _ => 4 }
            }

            OpCode::ReturnSub => {
                let pc = CPU::stack_pull_word(sys);
                sys.cpu.pc = pc;
                6
            }

            OpCode::ReturnInt => {
                let flags = CPU::stack_pull_byte(sys);
                sys.cpu.set_status(flags);
                let pc = CPU::stack_pull_word(sys);
                sys.cpu.pc = pc;
                pc_set = true;
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
                3
            }


            OpCode::PullAcc => {
                let value = CPU::stack_pull_byte(sys);
                sys.cpu.set_reg(&Register::A, value);
                sys.cpu.update_flags(value);
                4
            }

            OpCode::PushAcc => {
                CPU::stack_push_byte(sys, sys.cpu.a);
                4
            }
            
            OpCode::And => {
                let addr = get_addr(sys, &address_mode);
                sys.cpu.a &=  sys.read_byte(addr);
                sys.cpu.update_flags(sys.cpu.a);
                4 // TODO: Should take 2 - 6 cycles depending on addressing mode!
            }

            OpCode::Add => {
                let addr = get_addr(sys, &address_mode);
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
                // TODO: The carry use is probably not correct here

                // This instruction subtracts the contents of a memory location to the accumulator together with the not of the carry bit. 
                // If overflow occurs the carry bit is clear, this enables multiple byte subtraction to be performed.
                let addr = get_addr(sys, &address_mode);
                let sub = sys.read_byte(addr) as u16;
                let ack = sys.cpu.a as u16;

                let carry = if sys.cpu.carry {0} else {1};

                // let mut ack_lo = (ack & 0b1111) - (sub &0b1111) - carry;
                // if (ack_lo & 0b1_0000)!=0 { ack_lo -= 6; }

                // let mut ack_hi = (ack >> 4) - (sub >> 4) - (ack_lo & 0b1_0000);
                // if (ack_hi & 0b1_0000)!=0 { ack_hi -= 6; }

                // let value = ack - sub - carry;
                // sys.cpu.carry     = value & 0b1_0000_0000 != 0;
                // sys.cpu.zero      = value & 0b0_1111_1111 != 0;
                // sys.cpu.overflow  = ((value ^ sub) & 0b1000_0000 != 0) 
                //                     && (ack & sub) & 0b1000_0000 != 0;
                // sys.cpu.sign      = value & 0b1000_0000 != 0;
                // sys.cpu.a         = ((ack_hi << 4) | (ack_lo & 0b1111)) as u8;

                // let lhs_sign = (lhs & 0b10000000) != 0;
                // let rhs_sign = (rhs & 0b10000000) != 0;
                // let org_sign = lhs_sign != rhs_sign;


                // unsigned char Val = MemGet(CalcAddr);
                // let sub = sys.read_byte(addr) as u16;
                
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
                

                
                // let (value, overflow) = sys.cpu.a.overflowing_sub(rhs + carry);
                // sys.cpu.a = value;
                // sys.cpu.update_flags(value);

                // sys.cpu.overflow = if overflow {false} else {org_sign != sys.cpu.sign};

                // sys.cpu.carry = (lhs as u16 - rhs as u16 - carry as u16) & 256u16 != 0;

                // if overflow {
                //     sys.cpu.carry = false;
                // }

                // if !sys.cpu.carry {
                //     sys.cpu.carry = (org_sign != sys.cpu.sign);
                // }

                // eprintln!("{lhs} - {rhs} = {value} (overflow: {overflow})");
                eprintln!("{ack} - {sub} (- {carry}) = {result} ({result:016b})");
                0
            }
                        
            OpCode::Or => {
                let addr = get_addr(sys, &address_mode);
                sys.cpu.a |=  sys.read_byte(addr);
                sys.cpu.zero = sys.cpu.a == 0;
                sys.cpu.sign = (sys.cpu.a & 0b10000000) != 0;
                4 // TODO: Should take 2 - 6 cycles depending on addressing mode!
            }

                        
            OpCode::ExOr => {
                let addr = get_addr(sys, &address_mode);
                sys.cpu.a ^=  sys.read_byte(addr);
                sys.cpu.zero = sys.cpu.a == 0;
                sys.cpu.sign = (sys.cpu.a & 0b10000000) != 0;
                4 // TODO: Should take 2 - 6 cycles depending on addressing mode!
            }

            OpCode::Compare(reg) => {
                let addr = get_addr(sys, &address_mode);
                let value_m = sys.read_byte(addr);
                let value_r = sys.cpu.get_reg(reg);
                sys.cpu.carry = value_r >= value_m;
                let val = value_r.wrapping_sub(value_m);
                sys.cpu.update_flags(val);
                sys.cpu.zero = value_r == value_m;
                //sys.cpu.soft_break = true;
                4 // TODO: Should take 2 - 6 cycles depending on addressing mode!
            }

            OpCode::Inc(Some(reg)) => {
                let (value, _) = sys.cpu.get_reg(reg).overflowing_add(1);
                sys.cpu.set_reg(reg, value);
                sys.cpu.update_flags(value);
                2
            }

            OpCode::Inc(None) => {
                let addr = get_addr(sys, &address_mode);
                let (value, _) = sys.read_byte(addr).overflowing_add(1);
                sys.write_byte(addr, value);
                sys.cpu.update_flags(value);
                match address_mode {
                    AddressMode::Zero(None) => 5,
                    AddressMode::Absolute(Some(_)) => 7,
                    _ => 6 // Zero Page(X) and Absolute
                }
            }

            OpCode::Dec(Some(reg)) => {
                let (value, _) = sys.cpu.get_reg(reg).overflowing_sub(1);
                sys.cpu.set_reg(reg, value);
                sys.cpu.update_flags(value);
                2
            }

            OpCode::Dec(None) => {
                let addr = get_addr(sys, &address_mode);
                let (value, _) = sys.read_byte(addr).overflowing_sub(1);
                sys.write_byte(addr, value);
                sys.cpu.update_flags(value);
                match address_mode {
                    AddressMode::Zero(None) => 5,
                    AddressMode::Absolute(Some(_)) => 7,
                    _ => 6 // Zero Page(X) and Absolute
                }
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
                        let addr = get_addr(sys, &address_mode);
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
                        let addr = get_addr(sys, &address_mode);
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
                        let addr = get_addr(sys, &address_mode);
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
                        let addr = get_addr(sys, &address_mode);
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
                CPU::stack_push_word(sys, sys.cpu.pc);
                CPU::stack_push_byte(sys, sys.cpu.status());
                sys.cpu.pc = 0xfffe;
                sys.cpu.soft_break = true;

                7
            }

            op => panic!("opcode {op:?} is not implemented")
        };

        if !pc_set {
            sys.cpu.pc += 1;
        }

        cycles
    }
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