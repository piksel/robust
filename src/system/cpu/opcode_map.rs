use std::fmt::Display;

use crate::system::System;

use super::{CPU, opcode::OpCode, AddressMode, Flag, Register};


impl CPU {
    pub(crate) fn load(&self, sys: &System) -> (OpCode, AddressMode, u8) {
        let byte_code = sys.read_byte(self.pc);
        let (op, am) = match byte_code {
            
            // JMP
            0x4c => (OpCode::Jump, AddressMode::Absolute(None)),
            0x6c => (OpCode::Jump, AddressMode::Indirect(None)),

            // BCS, BEQ, BMI, BVS, Branch if flag is set
            0xb0 => (OpCode::BranchIf(Flag::Carry,    true), AddressMode::Relative),
            0xf0 => (OpCode::BranchIf(Flag::Zero,     true), AddressMode::Relative),
            0x30 => (OpCode::BranchIf(Flag::Negative, true), AddressMode::Relative),
            0x70 => (OpCode::BranchIf(Flag::Overflow, true), AddressMode::Relative),

            // BCC, BNE, BPL, Branch if flag is clear
            0x90 => (OpCode::BranchIf(Flag::Carry,    false), AddressMode::Relative),
            0xd0 => (OpCode::BranchIf(Flag::Zero,     false), AddressMode::Relative),
            0x10 => (OpCode::BranchIf(Flag::Negative, false), AddressMode::Relative),
            0x50 => (OpCode::BranchIf(Flag::Overflow, false), AddressMode::Relative),
            
            // JSR
            0x20 => (OpCode::JumpSub, AddressMode::Absolute(None)),

            // LDA
            0xa9 => (OpCode::Load(Register::A), AddressMode::Immediate),
            0xa5 => (OpCode::Load(Register::A), AddressMode::Zero(None)),
            0xb5 => (OpCode::Load(Register::A), AddressMode::Zero(Some(Register::X))),
            0xad => (OpCode::Load(Register::A), AddressMode::Absolute(None)),
            0xbd => (OpCode::Load(Register::A), AddressMode::Absolute(Some(Register::X))),
            0xb9 => (OpCode::Load(Register::A), AddressMode::Absolute(Some(Register::Y))),
            0xa1 => (OpCode::Load(Register::A), AddressMode::Indirect(Some(Register::X))),
            0xb1 => (OpCode::Load(Register::A), AddressMode::Indirect(Some(Register::Y))),

            // LDX
            0xa2 => (OpCode::Load(Register::X), AddressMode::Immediate),
            0xa6 => (OpCode::Load(Register::X), AddressMode::Zero(None)),
            0xb6 => (OpCode::Load(Register::X), AddressMode::Zero(Some(Register::Y))),
            0xae => (OpCode::Load(Register::X), AddressMode::Absolute(None)),
            0xbe => (OpCode::Load(Register::X), AddressMode::Absolute(Some(Register::Y))),

            // LDY
            0xa0 => (OpCode::Load(Register::Y), AddressMode::Immediate),
            0xa4 => (OpCode::Load(Register::Y), AddressMode::Zero(None)),
            0xb4 => (OpCode::Load(Register::Y), AddressMode::Zero(Some(Register::X))),
            0xac => (OpCode::Load(Register::Y), AddressMode::Absolute(None)),
            0xbc => (OpCode::Load(Register::Y), AddressMode::Absolute(Some(Register::X))),

            // LSR
            0x4a => (OpCode::ShiftRight, AddressMode::Register(Register::A)),
            0x46 => (OpCode::ShiftRight, AddressMode::Zero(None)),
            0x56 => (OpCode::ShiftRight, AddressMode::Zero(Some(Register::X))),
            0x4e => (OpCode::ShiftRight, AddressMode::Absolute(None)),
            0x5e => (OpCode::ShiftRight, AddressMode::Absolute(Some(Register::X))),

            // NOP, BRK
            0xea => (OpCode::NoOp, AddressMode::Implied),
            0x00 => (OpCode::Break, AddressMode::Implied),
            // INVALID
            0x04 => (OpCode::NoOp, AddressMode::Zero(None)),
            0x44 => (OpCode::NoOp, AddressMode::Zero(None)),
            0x64 => (OpCode::NoOp, AddressMode::Zero(None)),
            0x0c => (OpCode::NoOp, AddressMode::Absolute(None)),
            0x14 | 0x34 | 0x54 | 0x74 | 0xd4 | 0xf4
                 => (OpCode::NoOp, AddressMode::Zero(Some(Register::X))),
            0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xfa 
                 => (OpCode::NoOp, AddressMode::Implied),
            0x80 => (OpCode::NoOp, AddressMode::Immediate),
            0x1c | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc 
                 => (OpCode::NoOp, AddressMode::Absolute(Some(Register::X))),

            0xa3 => (OpCode::LoadHack(Register::A, Register::X), AddressMode::Indirect(Some(Register::X))),
            0xa7 => (OpCode::LoadHack(Register::A, Register::X), AddressMode::Zero(None)),
            0xaf => (OpCode::LoadHack(Register::A, Register::X), AddressMode::Absolute(None)),
            0xb3 => (OpCode::LoadHack(Register::A, Register::X), AddressMode::Indirect(Some(Register::Y))),
            0xb7 => (OpCode::LoadHack(Register::A, Register::X), AddressMode::Zero(Some(Register::Y))),
            0xbf => (OpCode::LoadHack(Register::A, Register::X), AddressMode::Absolute(Some(Register::Y))),

            0x83 => (OpCode::StoreHack(Register::A, Register::X), AddressMode::Indirect(Some(Register::X))),
            0x87 => (OpCode::StoreHack(Register::A, Register::X), AddressMode::Zero(None)),
            0x8f => (OpCode::StoreHack(Register::A, Register::X), AddressMode::Absolute(None)),
            0x93 => (OpCode::StoreHack(Register::A, Register::X), AddressMode::Indirect(Some(Register::Y))),
            0x97 => (OpCode::StoreHack(Register::A, Register::X), AddressMode::Zero(Some(Register::Y))),
            // SHA:
            // 0x9f => (OpCode::LoadHack(Register::A, Register::X), AddressMode::Absolute(Some(Register::Y))),

            0xeb => (OpCode::Sub, AddressMode::Immediate),

            0xc3 => (OpCode::DecHack, AddressMode::Indirect(Some(Register::X))),
            0xc7 => (OpCode::DecHack, AddressMode::Zero(None)),
            0xcf => (OpCode::DecHack, AddressMode::Absolute(None)),
            0xd3 => (OpCode::DecHack, AddressMode::Indirect(Some(Register::Y))),
            0xd7 => (OpCode::DecHack, AddressMode::Zero(Some(Register::X))),
            0xdb => (OpCode::DecHack, AddressMode::Absolute(Some(Register::Y))),
            0xdf => (OpCode::DecHack, AddressMode::Absolute(Some(Register::X))),

 
            // ORA
            0x09 => (OpCode::Or, AddressMode::Immediate),
            0x05 => (OpCode::Or, AddressMode::Zero(None)),
            0x15 => (OpCode::Or, AddressMode::Zero(Some(Register::X))),
            0x0d => (OpCode::Or, AddressMode::Absolute(None)),
            0x1d => (OpCode::Or, AddressMode::Absolute(Some(Register::X))),
            0x19 => (OpCode::Or, AddressMode::Absolute(Some(Register::Y))),
            0x01 => (OpCode::Or, AddressMode::Indirect(Some(Register::X))),
            0x11 => (OpCode::Or, AddressMode::Indirect(Some(Register::Y))),
            
            // EOR
            0x49 => (OpCode::ExOr, AddressMode::Immediate),
            0x45 => (OpCode::ExOr, AddressMode::Zero(None)),
            0x55 => (OpCode::ExOr, AddressMode::Zero(Some(Register::X))),
            0x4d => (OpCode::ExOr, AddressMode::Absolute(None)),
            0x5d => (OpCode::ExOr, AddressMode::Absolute(Some(Register::X))),
            0x59 => (OpCode::ExOr, AddressMode::Absolute(Some(Register::Y))),
            0x41 => (OpCode::ExOr, AddressMode::Indirect(Some(Register::X))),
            0x51 => (OpCode::ExOr, AddressMode::Indirect(Some(Register::Y))),
            
            // PHA, PHP, PLA, PLP
            0x48 => (OpCode::PushAcc,   AddressMode::Implied),
            0x08 => (OpCode::PushFlags, AddressMode::Implied),
            0x68 => (OpCode::PullAcc,   AddressMode::Implied),
            0x28 => (OpCode::PullFlags, AddressMode::Implied),

            // ROL
            0x2a => (OpCode::RotateLeft, AddressMode::Register(Register::A)),
            0x26 => (OpCode::RotateLeft, AddressMode::Zero(None)),
            0x36 => (OpCode::RotateLeft, AddressMode::Zero(Some(Register::X))),
            0x2e => (OpCode::RotateLeft, AddressMode::Absolute(None)),
            0x3e => (OpCode::RotateLeft, AddressMode::Absolute(Some(Register::X))),

            // ROR
            0x6a => (OpCode::RotateRight, AddressMode::Register(Register::A)),
            0x66 => (OpCode::RotateRight, AddressMode::Zero(None)),
            0x76 => (OpCode::RotateRight, AddressMode::Zero(Some(Register::X))),
            0x6e => (OpCode::RotateRight, AddressMode::Absolute(None)),
            0x7e => (OpCode::RotateRight, AddressMode::Absolute(Some(Register::X))),

            // RTI, RTS
            0x40 => (OpCode::ReturnInt, AddressMode::Implied),
            0x60 => (OpCode::ReturnSub, AddressMode::Implied),

            // SEC, SED, SEI, Set processor flags
            0x38 => (OpCode::SetFlag(Flag::Carry, true), AddressMode::Implied),
            0xf8 => (OpCode::SetFlag(Flag::Decimal, true), AddressMode::Implied),
            0x78 => (OpCode::SetFlag(Flag::Interrupt, true), AddressMode::Implied),

            // CLC, CLD, CLI, CLO, Clear processor flags
            0x18 => (OpCode::SetFlag(Flag::Carry, false), AddressMode::Implied),
            0xd8 => (OpCode::SetFlag(Flag::Decimal, false), AddressMode::Implied),
            0x58 => (OpCode::SetFlag(Flag::Interrupt, false), AddressMode::Implied),
            0xB8 => (OpCode::SetFlag(Flag::Overflow, false), AddressMode::Implied),


            // STA
            0x85 => (OpCode::Store(Register::A), AddressMode::Zero(None)),
            0x95 => (OpCode::Store(Register::A), AddressMode::Zero(Some(Register::X))),
            0x8d => (OpCode::Store(Register::A), AddressMode::Absolute(None)),
            0x9d => (OpCode::Store(Register::A), AddressMode::Absolute(Some(Register::X))),
            0x99 => (OpCode::Store(Register::A), AddressMode::Absolute(Some(Register::Y))),
            0x81 => (OpCode::Store(Register::A), AddressMode::Indirect(Some(Register::X))),
            0x91 => (OpCode::Store(Register::A), AddressMode::Indirect(Some(Register::Y))),

            // STX
            // 0x => (OpCode::Store(Register::X), AddressMode::Immediate),
            0x86 => (OpCode::Store(Register::X), AddressMode::Zero(None)),
            0x96 => (OpCode::Store(Register::X), AddressMode::Zero(Some(Register::Y))),
            0x8e => (OpCode::Store(Register::X), AddressMode::Absolute(None)),
            // 0x9e => (OpCode::Store(Register::X), AddressMode::Absolute(Some(Register::Y))),

            // STY
            // 0x => (OpCode::Store(Register::Y), AddressMode::Immediate),
            0x84 => (OpCode::Store(Register::Y), AddressMode::Zero(None)),
            0x94 => (OpCode::Store(Register::Y), AddressMode::Zero(Some(Register::X))),
            0x8c => (OpCode::Store(Register::Y), AddressMode::Absolute(None)),
            // 0x9c => (OpCode::Store(Register::Y), AddressMode::Absolute(Some(Register::X))),

            // BIT
            0x24 => (OpCode::Bit, AddressMode::Zero(None)),
            0x2c => (OpCode::Bit, AddressMode::Absolute(None)),

            // AND
            0x29 => (OpCode::And, AddressMode::Immediate),
            0x25 => (OpCode::And, AddressMode::Zero(None)),
            0x35 => (OpCode::And, AddressMode::Zero(Some(Register::X))),
            0x2d => (OpCode::And, AddressMode::Absolute(None)),
            0x3d => (OpCode::And, AddressMode::Absolute(Some(Register::X))),
            0x39 => (OpCode::And, AddressMode::Absolute(Some(Register::Y))),
            0x21 => (OpCode::And, AddressMode::Indirect(Some(Register::X))),
            0x31 => (OpCode::And, AddressMode::Indirect(Some(Register::Y))),

            // CMP
            0xc9 => (OpCode::Compare(Register::A), AddressMode::Immediate),
            0xc5 => (OpCode::Compare(Register::A), AddressMode::Zero(None)),
            0xd5 => (OpCode::Compare(Register::A), AddressMode::Zero(Some(Register::X))),
            0xcd => (OpCode::Compare(Register::A), AddressMode::Absolute(None)),
            0xdd => (OpCode::Compare(Register::A), AddressMode::Absolute(Some(Register::X))),
            0xd9 => (OpCode::Compare(Register::A), AddressMode::Absolute(Some(Register::Y))),
            0xc1 => (OpCode::Compare(Register::A), AddressMode::Indirect(Some(Register::X))),
            0xd1 => (OpCode::Compare(Register::A), AddressMode::Indirect(Some(Register::Y))),

            // CPX
            0xe0 => (OpCode::Compare(Register::X), AddressMode::Immediate),
            0xe4 => (OpCode::Compare(Register::X), AddressMode::Zero(None)),
            0xec => (OpCode::Compare(Register::X), AddressMode::Absolute(None)),

            // CPY
            0xc0 => (OpCode::Compare(Register::Y), AddressMode::Immediate),
            0xc4 => (OpCode::Compare(Register::Y), AddressMode::Zero(None)),
            0xcc => (OpCode::Compare(Register::Y), AddressMode::Absolute(None)),

            // ADC
            0x69 => (OpCode::Add, AddressMode::Immediate),
            0x65 => (OpCode::Add, AddressMode::Zero(None)),
            0x75 => (OpCode::Add, AddressMode::Zero(Some(Register::X))),
            0x6d => (OpCode::Add, AddressMode::Absolute(None)),
            0x7d => (OpCode::Add, AddressMode::Absolute(Some(Register::X))),
            0x79 => (OpCode::Add, AddressMode::Absolute(Some(Register::Y))),
            0x61 => (OpCode::Add, AddressMode::Indirect(Some(Register::X))),
            0x71 => (OpCode::Add, AddressMode::Indirect(Some(Register::Y))),

            // SBC
            0xe9 => (OpCode::Sub, AddressMode::Immediate),
            0xe5 => (OpCode::Sub, AddressMode::Zero(None)),
            0xf5 => (OpCode::Sub, AddressMode::Zero(Some(Register::X))),
            0xed => (OpCode::Sub, AddressMode::Absolute(None)),
            0xfd => (OpCode::Sub, AddressMode::Absolute(Some(Register::X))),
            0xf9 => (OpCode::Sub, AddressMode::Absolute(Some(Register::Y))),
            0xe1 => (OpCode::Sub, AddressMode::Indirect(Some(Register::X))),
            0xf1 => (OpCode::Sub, AddressMode::Indirect(Some(Register::Y))),

            // ASL
            0x0a => (OpCode::ShiftLeft, AddressMode::Register(Register::A)),
            0x06 => (OpCode::ShiftLeft, AddressMode::Zero(None)),
            0x16 => (OpCode::ShiftLeft, AddressMode::Zero(Some(Register::X))),
            0x0e => (OpCode::ShiftLeft, AddressMode::Absolute(None)),
            0x1e => (OpCode::ShiftLeft, AddressMode::Absolute(Some(Register::X))),

            // DEC
            0xc6 => (OpCode::Dec(None), AddressMode::Zero(None)),
            0xd6 => (OpCode::Dec(None), AddressMode::Zero(Some(Register::X))),
            0xce => (OpCode::Dec(None), AddressMode::Absolute(None)),
            0xde => (OpCode::Dec(None), AddressMode::Absolute(Some(Register::X))),

            // DEX, DEY
            0xca => (OpCode::Dec(Some(Register::X)), AddressMode::Implied),
            0x88 => (OpCode::Dec(Some(Register::Y)), AddressMode::Implied),

            // INC
            0xe6 => (OpCode::Inc(None), AddressMode::Zero(None)),
            0xf6 => (OpCode::Inc(None), AddressMode::Zero(Some(Register::X))),
            0xee => (OpCode::Inc(None), AddressMode::Absolute(None)),
            0xfe => (OpCode::Inc(None), AddressMode::Absolute(Some(Register::X))),

            // INX, INY
            0xe8 => (OpCode::Inc(Some(Register::X)), AddressMode::Implied),
            0xc8 => (OpCode::Inc(Some(Register::Y)), AddressMode::Implied),

            // TAX, TAY, TSX, TXA, TXS, TYA
            0xaa => (OpCode::Transfer(Register::A,  Register::X ), AddressMode::Implied),
            0xa8 => (OpCode::Transfer(Register::A,  Register::Y ), AddressMode::Implied),
            0xba => (OpCode::Transfer(Register::SP, Register::X ), AddressMode::Implied),
            0x8a => (OpCode::Transfer(Register::X,  Register::A ), AddressMode::Implied),
            0x9a => (OpCode::Transfer(Register::X,  Register::SP), AddressMode::Implied),
            0x98 => (OpCode::Transfer(Register::Y,  Register::A ), AddressMode::Implied),
                        
            instr => panic!("instruction not implemented: 0x{instr:02x}")
        };

        (op, am, byte_code)
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