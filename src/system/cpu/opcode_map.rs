use crate::system::System;
use crate::system::cpu::{self, opcode::OpCode, AddressMode, Flag, Register};


pub(crate) fn load(sys: &mut System) -> anyhow::Result<(OpCode, AddressMode)> {
    match cpu::shift_pc(sys)? {
        
        // JMP
        0x4c => Ok((OpCode::Jump, AddressMode::Absolute(None))),
        0x6c => Ok((OpCode::Jump, AddressMode::Indirect(None))),

        // BCS, BEQ, BMI, BVS, Branch if flag is set
        0xb0 => Ok((OpCode::BranchIf(Flag::Carry,    true), AddressMode::Relative)),
        0xf0 => Ok((OpCode::BranchIf(Flag::Zero,     true), AddressMode::Relative)),
        0x30 => Ok((OpCode::BranchIf(Flag::Negative, true), AddressMode::Relative)),
        0x70 => Ok((OpCode::BranchIf(Flag::Overflow, true), AddressMode::Relative)),

        // BCC, BNE, BPL, Branch if flag is clear
        0x90 => Ok((OpCode::BranchIf(Flag::Carry,    false), AddressMode::Relative)),
        0xd0 => Ok((OpCode::BranchIf(Flag::Zero,     false), AddressMode::Relative)),
        0x10 => Ok((OpCode::BranchIf(Flag::Negative, false), AddressMode::Relative)),
        0x50 => Ok((OpCode::BranchIf(Flag::Overflow, false), AddressMode::Relative)),
        
        // JSR
        0x20 => Ok((OpCode::JumpSub, AddressMode::Absolute(None))),

        // LDA
        0xa9 => Ok((OpCode::Load(Register::A), AddressMode::Immediate)),
        0xa5 => Ok((OpCode::Load(Register::A), AddressMode::Zero(None))),
        0xb5 => Ok((OpCode::Load(Register::A), AddressMode::Zero(Some(Register::X)))),
        0xad => Ok((OpCode::Load(Register::A), AddressMode::Absolute(None))),
        0xbd => Ok((OpCode::Load(Register::A), AddressMode::Absolute(Some(Register::X)))),
        0xb9 => Ok((OpCode::Load(Register::A), AddressMode::Absolute(Some(Register::Y)))),
        0xa1 => Ok((OpCode::Load(Register::A), AddressMode::Indirect(Some(Register::X)))),
        0xb1 => Ok((OpCode::Load(Register::A), AddressMode::Indirect(Some(Register::Y)))),

        // LDX
        0xa2 => Ok((OpCode::Load(Register::X), AddressMode::Immediate)),
        0xa6 => Ok((OpCode::Load(Register::X), AddressMode::Zero(None))),
        0xb6 => Ok((OpCode::Load(Register::X), AddressMode::Zero(Some(Register::Y)))),
        0xae => Ok((OpCode::Load(Register::X), AddressMode::Absolute(None))),
        0xbe => Ok((OpCode::Load(Register::X), AddressMode::Absolute(Some(Register::Y)))),

        // LDY
        0xa0 => Ok((OpCode::Load(Register::Y), AddressMode::Immediate)),
        0xa4 => Ok((OpCode::Load(Register::Y), AddressMode::Zero(None))),
        0xb4 => Ok((OpCode::Load(Register::Y), AddressMode::Zero(Some(Register::X)))),
        0xac => Ok((OpCode::Load(Register::Y), AddressMode::Absolute(None))),
        0xbc => Ok((OpCode::Load(Register::Y), AddressMode::Absolute(Some(Register::X)))),

        // LSR
        0x4a => Ok((OpCode::ShiftRight, AddressMode::Register(Register::A))),
        0x46 => Ok((OpCode::ShiftRight, AddressMode::Zero(None))),
        0x56 => Ok((OpCode::ShiftRight, AddressMode::Zero(Some(Register::X)))),
        0x4e => Ok((OpCode::ShiftRight, AddressMode::Absolute(None))),
        0x5e => Ok((OpCode::ShiftRight, AddressMode::Absolute(Some(Register::X)))),

        // NOP, BRK
        0xea => Ok((OpCode::NoOp, AddressMode::Implied)),
        0x00 => Ok((OpCode::Break, AddressMode::Implied)),
       
        // INVALID
        0x04 => Ok((OpCode::NoOp, AddressMode::Zero(None))),
        0x44 => Ok((OpCode::NoOp, AddressMode::Zero(None))),
        0x64 => Ok((OpCode::NoOp, AddressMode::Zero(None))),
        0x0c => Ok((OpCode::NoOp, AddressMode::Absolute(None))),
        0x14 | 0x34 | 0x54 | 0x74 | 0xd4 | 0xf4
                => Ok((OpCode::NoOp, AddressMode::Zero(Some(Register::X)))),
        0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xfa 
                => Ok((OpCode::NoOp, AddressMode::Implied)),
        0x80 => Ok((OpCode::NoOp, AddressMode::Immediate)),
        0x1c | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc 
                => Ok((OpCode::NoOp, AddressMode::Absolute(Some(Register::X)))),

        0xa3 => Ok((OpCode::LoadHack(Register::A, Register::X), AddressMode::Indirect(Some(Register::X)))),
        0xa7 => Ok((OpCode::LoadHack(Register::A, Register::X), AddressMode::Zero(None))),
        0xaf => Ok((OpCode::LoadHack(Register::A, Register::X), AddressMode::Absolute(None))),
        0xb3 => Ok((OpCode::LoadHack(Register::A, Register::X), AddressMode::Indirect(Some(Register::Y)))),
        0xb7 => Ok((OpCode::LoadHack(Register::A, Register::X), AddressMode::Zero(Some(Register::Y)))),
        0xbf => Ok((OpCode::LoadHack(Register::A, Register::X), AddressMode::Absolute(Some(Register::Y)))),

        0x83 => Ok((OpCode::StoreHack(Register::A, Register::X), AddressMode::Indirect(Some(Register::X)))),
        0x87 => Ok((OpCode::StoreHack(Register::A, Register::X), AddressMode::Zero(None))),
        0x8f => Ok((OpCode::StoreHack(Register::A, Register::X), AddressMode::Absolute(None))),
        0x93 => Ok((OpCode::StoreHack(Register::A, Register::X), AddressMode::Indirect(Some(Register::Y)))),
        0x97 => Ok((OpCode::StoreHack(Register::A, Register::X), AddressMode::Zero(Some(Register::Y)))),
        // SHA:
        /*
        Stores A AND X AND (high-byte of addr. + 1) at addr.
        unstable: sometimes 'AND (H+1)' is dropped, page boundary crossings may not work 
                  (with the high-byte of the value used as the high-byte of the address)
        */
        // 0x9f => Ok((OpCode::LoadHack(Register::A, Register::X), AddressMode::Absolute(Some(Register::Y)))),

        // Nop + Sub 🤮
        0xeb => Ok((OpCode::Sub, AddressMode::Immediate)),

        // Inc + Sub
        0xe3 => Ok((OpCode::IncSubHack, AddressMode::Indirect(Some(Register::X)))),
        0xe7 => Ok((OpCode::IncSubHack, AddressMode::Zero(None))),
        0xef => Ok((OpCode::IncSubHack, AddressMode::Absolute(None))),
        0xf3 => Ok((OpCode::IncSubHack, AddressMode::Indirect(Some(Register::Y)))),
        0xf7 => Ok((OpCode::IncSubHack, AddressMode::Zero(Some(Register::X)))),
        0xfb => Ok((OpCode::IncSubHack, AddressMode::Absolute(Some(Register::Y)))),
        0xff => Ok((OpCode::IncSubHack, AddressMode::Absolute(Some(Register::X)))),

        // Dec + Cmp
        0xc3 => Ok((OpCode::DecCmpHack, AddressMode::Indirect(Some(Register::X)))),
        0xc7 => Ok((OpCode::DecCmpHack, AddressMode::Zero(None))),
        0xcf => Ok((OpCode::DecCmpHack, AddressMode::Absolute(None))),
        0xd3 => Ok((OpCode::DecCmpHack, AddressMode::Indirect(Some(Register::Y)))),
        0xd7 => Ok((OpCode::DecCmpHack, AddressMode::Zero(Some(Register::X)))),
        0xdb => Ok((OpCode::DecCmpHack, AddressMode::Absolute(Some(Register::Y)))),
        0xdf => Ok((OpCode::DecCmpHack, AddressMode::Absolute(Some(Register::X)))),

        // Asl + Ora
        0x03 => Ok((OpCode::ShiftLeftOrHack, AddressMode::Indirect(Some(Register::X)))),
        0x07 => Ok((OpCode::ShiftLeftOrHack, AddressMode::Zero(None))),
        0x0f => Ok((OpCode::ShiftLeftOrHack, AddressMode::Absolute(None))),
        0x13 => Ok((OpCode::ShiftLeftOrHack, AddressMode::Indirect(Some(Register::Y)))),
        0x17 => Ok((OpCode::ShiftLeftOrHack, AddressMode::Zero(Some(Register::X)))),
        0x1b => Ok((OpCode::ShiftLeftOrHack, AddressMode::Absolute(Some(Register::Y)))),
        0x1f => Ok((OpCode::ShiftLeftOrHack, AddressMode::Absolute(Some(Register::X)))),

        // Rol + And
        0x23 => Ok((OpCode::RotLeftAndHack, AddressMode::Indirect(Some(Register::X)))),
        0x27 => Ok((OpCode::RotLeftAndHack, AddressMode::Zero(None))),
        0x2f => Ok((OpCode::RotLeftAndHack, AddressMode::Absolute(None))),
        0x33 => Ok((OpCode::RotLeftAndHack, AddressMode::Indirect(Some(Register::Y)))),
        0x37 => Ok((OpCode::RotLeftAndHack, AddressMode::Zero(Some(Register::X)))),
        0x3b => Ok((OpCode::RotLeftAndHack, AddressMode::Absolute(Some(Register::Y)))),
        0x3f => Ok((OpCode::RotLeftAndHack, AddressMode::Absolute(Some(Register::X)))),

        // Lsr + Eor, M = 0 -> [76543210] -> C, A EOR M -> A
        0x43 => Ok((OpCode::ShiftRightOrHack, AddressMode::Indirect(Some(Register::X)))),
        0x47 => Ok((OpCode::ShiftRightOrHack, AddressMode::Zero(None))),
        0x4f => Ok((OpCode::ShiftRightOrHack, AddressMode::Absolute(None))),
        0x53 => Ok((OpCode::ShiftRightOrHack, AddressMode::Indirect(Some(Register::Y)))),
        0x57 => Ok((OpCode::ShiftRightOrHack, AddressMode::Zero(Some(Register::X)))),
        0x5b => Ok((OpCode::ShiftRightOrHack, AddressMode::Absolute(Some(Register::Y)))),
        0x5f => Ok((OpCode::ShiftRightOrHack, AddressMode::Absolute(Some(Register::X)))),

        // Ror + Adc, M = C -> [76543210] -> C, A + M + C -> A, C
        0x63 => Ok((OpCode::RotRightAddHack, AddressMode::Indirect(Some(Register::X)))),
        0x67 => Ok((OpCode::RotRightAddHack, AddressMode::Zero(None))),
        0x6f => Ok((OpCode::RotRightAddHack, AddressMode::Absolute(None))),
        0x73 => Ok((OpCode::RotRightAddHack, AddressMode::Indirect(Some(Register::Y)))),
        0x77 => Ok((OpCode::RotRightAddHack, AddressMode::Zero(Some(Register::X)))),
        0x7b => Ok((OpCode::RotRightAddHack, AddressMode::Absolute(Some(Register::Y)))),
        0x7f => Ok((OpCode::RotRightAddHack, AddressMode::Absolute(Some(Register::X)))),


        // --- END OF INVALID ----------------------------------------------------

        // ORA
        0x09 => Ok((OpCode::Or, AddressMode::Immediate)),
        0x05 => Ok((OpCode::Or, AddressMode::Zero(None))),
        0x15 => Ok((OpCode::Or, AddressMode::Zero(Some(Register::X)))),
        0x0d => Ok((OpCode::Or, AddressMode::Absolute(None))),
        0x1d => Ok((OpCode::Or, AddressMode::Absolute(Some(Register::X)))),
        0x19 => Ok((OpCode::Or, AddressMode::Absolute(Some(Register::Y)))),
        0x01 => Ok((OpCode::Or, AddressMode::Indirect(Some(Register::X)))),
        0x11 => Ok((OpCode::Or, AddressMode::Indirect(Some(Register::Y)))),
        
        // EOR
        0x49 => Ok((OpCode::ExOr, AddressMode::Immediate)),
        0x45 => Ok((OpCode::ExOr, AddressMode::Zero(None))),
        0x55 => Ok((OpCode::ExOr, AddressMode::Zero(Some(Register::X)))),
        0x4d => Ok((OpCode::ExOr, AddressMode::Absolute(None))),
        0x5d => Ok((OpCode::ExOr, AddressMode::Absolute(Some(Register::X)))),
        0x59 => Ok((OpCode::ExOr, AddressMode::Absolute(Some(Register::Y)))),
        0x41 => Ok((OpCode::ExOr, AddressMode::Indirect(Some(Register::X)))),
        0x51 => Ok((OpCode::ExOr, AddressMode::Indirect(Some(Register::Y)))),
        
        // PHA, PHP, PLA, PLP
        0x48 => Ok((OpCode::PushAcc,   AddressMode::Implied)),
        0x08 => Ok((OpCode::PushFlags, AddressMode::Implied)),
        0x68 => Ok((OpCode::PullAcc,   AddressMode::Implied)),
        0x28 => Ok((OpCode::PullFlags, AddressMode::Implied)),

        // ROL
        0x2a => Ok((OpCode::RotateLeft, AddressMode::Register(Register::A))),
        0x26 => Ok((OpCode::RotateLeft, AddressMode::Zero(None))),
        0x36 => Ok((OpCode::RotateLeft, AddressMode::Zero(Some(Register::X)))),
        0x2e => Ok((OpCode::RotateLeft, AddressMode::Absolute(None))),
        0x3e => Ok((OpCode::RotateLeft, AddressMode::Absolute(Some(Register::X)))),

        // ROR
        0x6a => Ok((OpCode::RotateRight, AddressMode::Register(Register::A))),
        0x66 => Ok((OpCode::RotateRight, AddressMode::Zero(None))),
        0x76 => Ok((OpCode::RotateRight, AddressMode::Zero(Some(Register::X)))),
        0x6e => Ok((OpCode::RotateRight, AddressMode::Absolute(None))),
        0x7e => Ok((OpCode::RotateRight, AddressMode::Absolute(Some(Register::X)))),

        // RTI, RTS
        0x40 => Ok((OpCode::ReturnInt, AddressMode::Implied)),
        0x60 => Ok((OpCode::ReturnSub, AddressMode::Implied)),

        // SEC, SED, SEI, Set processor flags
        0x38 => Ok((OpCode::SetFlag(Flag::Carry, true), AddressMode::Implied)),
        0xf8 => Ok((OpCode::SetFlag(Flag::Decimal, true), AddressMode::Implied)),
        0x78 => Ok((OpCode::SetFlag(Flag::Interrupt, true), AddressMode::Implied)),

        // CLC, CLD, CLI, CLO, Clear processor flags
        0x18 => Ok((OpCode::SetFlag(Flag::Carry, false), AddressMode::Implied)),
        0xd8 => Ok((OpCode::SetFlag(Flag::Decimal, false), AddressMode::Implied)),
        0x58 => Ok((OpCode::SetFlag(Flag::Interrupt, false), AddressMode::Implied)),
        0xB8 => Ok((OpCode::SetFlag(Flag::Overflow, false), AddressMode::Implied)),


        // STA
        0x85 => Ok((OpCode::Store(Register::A), AddressMode::Zero(None))),
        0x95 => Ok((OpCode::Store(Register::A), AddressMode::Zero(Some(Register::X)))),
        0x8d => Ok((OpCode::Store(Register::A), AddressMode::Absolute(None))),
        0x9d => Ok((OpCode::Store(Register::A), AddressMode::Absolute(Some(Register::X)))),
        0x99 => Ok((OpCode::Store(Register::A), AddressMode::Absolute(Some(Register::Y)))),
        0x81 => Ok((OpCode::Store(Register::A), AddressMode::Indirect(Some(Register::X)))),
        0x91 => Ok((OpCode::Store(Register::A), AddressMode::Indirect(Some(Register::Y)))),

        // STX
        // 0x => Ok((OpCode::Store(Register::X), AddressMode::Immediate)),
        0x86 => Ok((OpCode::Store(Register::X), AddressMode::Zero(None))),
        0x96 => Ok((OpCode::Store(Register::X), AddressMode::Zero(Some(Register::Y)))),
        0x8e => Ok((OpCode::Store(Register::X), AddressMode::Absolute(None))),
        // 0x9e => Ok((OpCode::Store(Register::X), AddressMode::Absolute(Some(Register::Y)))),

        // STY
        // 0x => Ok((OpCode::Store(Register::Y), AddressMode::Immediate)),
        0x84 => Ok((OpCode::Store(Register::Y), AddressMode::Zero(None))),
        0x94 => Ok((OpCode::Store(Register::Y), AddressMode::Zero(Some(Register::X)))),
        0x8c => Ok((OpCode::Store(Register::Y), AddressMode::Absolute(None))),
        // 0x9c => Ok((OpCode::Store(Register::Y), AddressMode::Absolute(Some(Register::X)))),

        // BIT
        0x24 => Ok((OpCode::Bit, AddressMode::Zero(None))),
        0x2c => Ok((OpCode::Bit, AddressMode::Absolute(None))),

        // AND
        0x29 => Ok((OpCode::And, AddressMode::Immediate)),
        0x25 => Ok((OpCode::And, AddressMode::Zero(None))),
        0x35 => Ok((OpCode::And, AddressMode::Zero(Some(Register::X)))),
        0x2d => Ok((OpCode::And, AddressMode::Absolute(None))),
        0x3d => Ok((OpCode::And, AddressMode::Absolute(Some(Register::X)))),
        0x39 => Ok((OpCode::And, AddressMode::Absolute(Some(Register::Y)))),
        0x21 => Ok((OpCode::And, AddressMode::Indirect(Some(Register::X)))),
        0x31 => Ok((OpCode::And, AddressMode::Indirect(Some(Register::Y)))),

        // CMP
        0xc9 => Ok((OpCode::Compare(Register::A), AddressMode::Immediate)),
        0xc5 => Ok((OpCode::Compare(Register::A), AddressMode::Zero(None))),
        0xd5 => Ok((OpCode::Compare(Register::A), AddressMode::Zero(Some(Register::X)))),
        0xcd => Ok((OpCode::Compare(Register::A), AddressMode::Absolute(None))),
        0xdd => Ok((OpCode::Compare(Register::A), AddressMode::Absolute(Some(Register::X)))),
        0xd9 => Ok((OpCode::Compare(Register::A), AddressMode::Absolute(Some(Register::Y)))),
        0xc1 => Ok((OpCode::Compare(Register::A), AddressMode::Indirect(Some(Register::X)))),
        0xd1 => Ok((OpCode::Compare(Register::A), AddressMode::Indirect(Some(Register::Y)))),

        // CPX
        0xe0 => Ok((OpCode::Compare(Register::X), AddressMode::Immediate)),
        0xe4 => Ok((OpCode::Compare(Register::X), AddressMode::Zero(None))),
        0xec => Ok((OpCode::Compare(Register::X), AddressMode::Absolute(None))),

        // CPY
        0xc0 => Ok((OpCode::Compare(Register::Y), AddressMode::Immediate)),
        0xc4 => Ok((OpCode::Compare(Register::Y), AddressMode::Zero(None))),
        0xcc => Ok((OpCode::Compare(Register::Y), AddressMode::Absolute(None))),

        // ADC
        0x69 => Ok((OpCode::Add, AddressMode::Immediate)),
        0x65 => Ok((OpCode::Add, AddressMode::Zero(None))),
        0x75 => Ok((OpCode::Add, AddressMode::Zero(Some(Register::X)))),
        0x6d => Ok((OpCode::Add, AddressMode::Absolute(None))),
        0x7d => Ok((OpCode::Add, AddressMode::Absolute(Some(Register::X)))),
        0x79 => Ok((OpCode::Add, AddressMode::Absolute(Some(Register::Y)))),
        0x61 => Ok((OpCode::Add, AddressMode::Indirect(Some(Register::X)))),
        0x71 => Ok((OpCode::Add, AddressMode::Indirect(Some(Register::Y)))),

        // SBC
        0xe9 => Ok((OpCode::Sub, AddressMode::Immediate)),
        0xe5 => Ok((OpCode::Sub, AddressMode::Zero(None))),
        0xf5 => Ok((OpCode::Sub, AddressMode::Zero(Some(Register::X)))),
        0xed => Ok((OpCode::Sub, AddressMode::Absolute(None))),
        0xfd => Ok((OpCode::Sub, AddressMode::Absolute(Some(Register::X)))),
        0xf9 => Ok((OpCode::Sub, AddressMode::Absolute(Some(Register::Y)))),
        0xe1 => Ok((OpCode::Sub, AddressMode::Indirect(Some(Register::X)))),
        0xf1 => Ok((OpCode::Sub, AddressMode::Indirect(Some(Register::Y)))),

        // ASL
        0x0a => Ok((OpCode::ShiftLeft, AddressMode::Register(Register::A))),
        0x06 => Ok((OpCode::ShiftLeft, AddressMode::Zero(None))),
        0x16 => Ok((OpCode::ShiftLeft, AddressMode::Zero(Some(Register::X)))),
        0x0e => Ok((OpCode::ShiftLeft, AddressMode::Absolute(None))),
        0x1e => Ok((OpCode::ShiftLeft, AddressMode::Absolute(Some(Register::X)))),

        // DEC
        0xc6 => Ok((OpCode::Dec(None), AddressMode::Zero(None))),
        0xd6 => Ok((OpCode::Dec(None), AddressMode::Zero(Some(Register::X)))),
        0xce => Ok((OpCode::Dec(None), AddressMode::Absolute(None))),
        0xde => Ok((OpCode::Dec(None), AddressMode::Absolute(Some(Register::X)))),

        // DEX, DEY
        0xca => Ok((OpCode::Dec(Some(Register::X)), AddressMode::Implied)),
        0x88 => Ok((OpCode::Dec(Some(Register::Y)), AddressMode::Implied)),

        // INC
        0xe6 => Ok((OpCode::Inc(None), AddressMode::Zero(None))),
        0xf6 => Ok((OpCode::Inc(None), AddressMode::Zero(Some(Register::X)))),
        0xee => Ok((OpCode::Inc(None), AddressMode::Absolute(None))),
        0xfe => Ok((OpCode::Inc(None), AddressMode::Absolute(Some(Register::X)))),

        // INX, INY
        0xe8 => Ok((OpCode::Inc(Some(Register::X)), AddressMode::Implied)),
        0xc8 => Ok((OpCode::Inc(Some(Register::Y)), AddressMode::Implied)),

        // TAX, TAY, TSX, TXA, TXS, TYA
        0xaa => Ok((OpCode::Transfer(Register::A,  Register::X ), AddressMode::Implied)),
        0xa8 => Ok((OpCode::Transfer(Register::A,  Register::Y ), AddressMode::Implied)),
        0xba => Ok((OpCode::Transfer(Register::SP, Register::X ), AddressMode::Implied)),
        0x8a => Ok((OpCode::Transfer(Register::X,  Register::A ), AddressMode::Implied)),
        0x9a => Ok((OpCode::Transfer(Register::X,  Register::SP), AddressMode::Implied)),
        0x98 => Ok((OpCode::Transfer(Register::Y,  Register::A ), AddressMode::Implied)),
                    
        instr => anyhow::bail!("instruction not implemented: 0x{instr:02x}")
    }
}


// impl Display for OpCode {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self
//     }
// }

pub fn format_op_byte(byte: u8) -> &'static str {
    match byte {
        0 => "BRK",
        1 | 5 | 9 | 13 | 17 | 21 | 25 | 29 => "ORA",
        2 | 18 | 34 | 50 | 66 | 82 | 98 | 114 | 130 | 146 | 178 | 194 | 210 | 226 | 242 => "XXX",
        3 | 7 | 15 | 19 | 23 | 27 | 31 => "SLO",
        4 | 12 | 20 | 26 | 28 | 52 | 58 | 60 | 68 | 84 | 90 | 92 | 100 | 116 | 122 | 124 | 128 | 137 | 212 | 218 | 220 | 234 | 244 | 250 | 252 => "NOP",
        6 | 10 | 14 | 22 | 30 => "ASL",
        8 => "PHP",
        11 | 43 => "ANC",
        16 => "BPL",
        24 => "CLC",
        32 => "JSR",
        33 | 37 | 41 | 45 | 49 | 53 | 57 | 61 => "AND",
        35 | 39 | 47 | 51 | 55 | 59 | 63 => "RLA",
        36 | 44 => "BIT",
        38 | 42 | 46 | 54 | 62 => "ROL",
        40 => "PLP",
        48 => "BMI",
        56 => "SEC",
        64 => "RTI",
        65 | 69 | 73 | 77 | 81 | 85 | 89 | 93 => "EOR",
        67 | 71 | 79 | 83 | 87 | 91 | 95 => "SRE",
        70 | 74 | 78 | 86 | 94 => "LSR",
        72 => "PHA",
        75 => "ASR",
        76 | 108 => "JMP",
        80 => "BVC",
        88 => "CLI",
        96 => "RTS",
        97 | 101 | 105 | 109 | 113 | 117 | 121 | 125 => "ADC",
        99 | 103 | 111 | 115 | 119 | 123 | 127 => "RRA",
        102 | 106 | 110 | 118 | 126 => "ROR",
        104 => "PLA",
        107 => "ARR",
        112 => "BVS",
        120 => "SEI",
        129 | 133 | 141 | 145 | 149 | 153 | 157 => "STA",
        131 | 135 | 143 | 151 => "SAX",
        132 | 140 | 148 => "STY",
        134 | 142 | 150 => "STX",
        136 => "DEY",
        138 => "TXA",
        139 => "ANE",
        144 => "BCC",
        147 | 159 => "SHA",
        152 => "TYA",
        154 => "TXS",
        155 => "SHS",
        156 => "SHY",
        158 => "SHX",
        160 | 164 | 172 | 180 | 188 => "LDY",
        161 | 165 | 169 | 173 | 177 | 181 | 185 | 189 => "LDA",
        162 | 166 | 174 | 182 | 190 => "LDX",
        163 | 167 | 175 | 179 | 183 | 191 => "LAX",
        168 => "TAY",
        170 => "TAX",
        171 => "LXA",
        176 => "BCS",
        184 => "CLV",
        186 => "TSX",
        187 => "LAS",
        192 | 196 | 204 => "CPY",
        193 | 197 | 201 | 205 | 209 | 213 | 217 | 221 => "CMP",
        195 | 199 | 207 | 211 | 215 | 219 | 223 => "DCP",
        198 | 206 | 214 | 222 => "DEC",
        200 => "INY",
        202 => "DEX",
        203 => "SBX",
        208 => "BNE",
        216 => "CLD",
        224 | 228 | 236 => "CPX",
        225 | 229 | 233 | 235 | 237 | 241 | 245 | 249 | 253 => "SBC",
        227 | 231 | 239 | 243 | 247 | 251 | 255 => "ISB",
        230 | 238 | 246 | 254 => "INC",
        232 => "INX",
        240 => "BEQ",
        248 => "SED",
    }
}