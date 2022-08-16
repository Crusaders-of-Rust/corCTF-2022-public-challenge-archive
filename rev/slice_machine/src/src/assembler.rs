use std::{collections::HashMap, sync::atomic::{AtomicUsize, Ordering}};


#[derive(Debug)]
pub enum Instruction {
    Add(u8,u8),
    And(u8,u8),
    Xor(u8,u8),
    Or(u8,u8),

    AddI(u8,u16),
    AndI(u8,u16),
    XorI(u8,u16),
    OrI(u8,u16),

    Load(u8,u8),
    Store(u8,u8),
    LoadI(u8,u16),
    StoreI(u8,u16),

    Jump(u16),
    JumpR(u8),
    Jeq(u8,u8,u16),
    Input,
    Output,

    OrLabel(u8,String),
    JumpLabel(String),
    JeqLabel(u8,u8,String),
    LoadLabel(u8, String),
    StoreLabel(u8, String),
    Label(String),

    Data(Vec<bool>)
}

macro_rules! bits {
    ( $($b:expr),* ) => {
        {
            let mut v: Vec<bool> = Vec::new();
            $(
                {
                    v.extend_from_slice($b);
                }
            )*
            v
        }
    };
}

macro_rules! op {
    ($o:expr) => {
        &[
            ($o >> 3) & 1 == 1,
            ($o >> 2) & 1 == 1,
            ($o >> 1) & 1 == 1,
            ($o >> 0) & 1 == 1,
        ]
    };
}

macro_rules! num {
    ($r:expr, $N:expr) => {
        &(0..$N).map(|i| ($r >> i) & 1 == 1)
            .collect::<Vec<bool>>()
    };
}

macro_rules! rtype {
    ($o:expr, $ra:expr, $rb:expr) => {
        bits!(op!($o), num!($ra,3), num!($rb,3))
    };
}

macro_rules! itype {
    ($o:expr, $ra:expr, $imm:expr) => {
        bits!(op!($o), num!($ra,3), num!($imm,16))
    };
}

macro_rules! btype {
    ($o:expr, $ra:expr, $rb:expr, $imm:expr) => {
        bits!(op!($o), num!($ra,3), num!($rb,3), num!($imm,16))
    };
}

macro_rules! ztype {
    ($o:expr, $d:expr) => {
        bits!(op!($o), num!($d,1))
    };
}

impl Instruction {
    pub fn bits(&self) -> Vec<bool> {
        match self {
            Instruction::Add(ra,rb) => rtype!(0b0000, ra, rb),
            Instruction::Xor(ra,rb) => rtype!(0b0001, ra, rb),
            Instruction::And(ra,rb) => rtype!(0b0010, ra, rb),
            Instruction::Or(ra,rb) => rtype!(0b0011, ra, rb),

            Instruction::AddI(ra,imm) => itype!(0b0100, ra, imm),
            Instruction::XorI(ra,imm) => itype!(0b0101, ra, imm),
            Instruction::AndI(ra,imm) => itype!(0b0110, ra, imm),
            Instruction::OrI(ra,imm) => itype!(0b0111, ra, imm),

            Instruction::Load(ra, rb) => rtype!(0b1000, ra, rb),
            Instruction::Store(ra, rb) => rtype!(0b1001, ra, rb),
            Instruction::LoadI(ra, imm) => itype!(0b1010, ra, imm),
            Instruction::StoreI(ra, imm) => itype!(0b1011, ra, imm),

            Instruction::Jump(imm) => btype!(0b1100, 0, 0, imm),
            Instruction::JumpR(ra) => btype!(0b1101, ra, 0, 0),
            Instruction::Jeq(ra, rb, imm) => btype!(0b1110, ra, rb, imm),

            Instruction::Input => ztype!(0b1111, 1),
            Instruction::Output => ztype!(0b1111, 0),

            // Will be replaced
            Instruction::OrLabel(_,_) => itype!(0b0111, 0, 0),
            Instruction::JumpLabel(_) => btype!(0b1100, 0, 0, 0),
            Instruction::JeqLabel(_,_,_) => btype!(0b1110, 0, 0, 0),
            Instruction::LoadLabel(ra, _) => itype!(0b1010, ra, 0),
            Instruction::StoreLabel(ra, _) => itype!(0b1011, ra, 0),
            Instruction::Label(_) => Vec::new(),

            Instruction::Data(v) => v.clone(),
        }
    }
}

#[derive(Clone)]
pub enum ValType<'a> {
    Reg(u8),
    Imm(u16),
    Label(&'a str)
}

impl ValType<'_> {
    pub fn parse(p: &str) -> ValType {
        match &p[0..1] {
            "r" => {
                let v: u8 = p[1..].parse().unwrap();
                assert!(v <= 7);
                ValType::Reg(v)
            },
            "\'" => {
                ValType::Label(&p[1..])
            }
            _ => {
                ValType::Imm(p.parse().unwrap())
            }
        }
    }
}

pub fn encode(instr: &[Instruction]) -> Vec<bool> {
    let mut label_map: HashMap<String, usize> = HashMap::new();

    let mut pos = 0usize;
    for it in instr.iter() {
        match it {
            Instruction::Label(lab) => {
                label_map.insert(lab.clone(), pos);
            }
            _ => {
                pos += it.bits().len();
            }
        }
    }

    println!("Labels: {:#?}", label_map);

    let mut out = Vec::new();
    for it in instr.iter() {
        let bits = match it {
            Instruction::OrLabel(ra, lab) => {
                let target = label_map.get(lab).unwrap();
                Instruction::OrI(*ra, *target as u16).bits()
            },
            Instruction::JumpLabel(lab) => {
                println!("lab {:?}", lab);
                let target = label_map.get(lab).unwrap();
                Instruction::Jump(*target as u16).bits()
            },
            Instruction::JeqLabel(ra, rb, lab) => {
                let target = label_map.get(lab).unwrap();
                Instruction::Jeq(*ra, *rb, *target as u16).bits()
            },
            Instruction::LoadLabel(ra, lab) => {
                let target = label_map.get(lab).unwrap();
                Instruction::LoadI(*ra, *target as u16).bits()
            },
            Instruction::StoreLabel(ra, lab) => {
                let target = label_map.get(lab).unwrap();
                Instruction::StoreI(*ra, *target as u16).bits()
            },
            x => x.bits()
        };

        out.extend_from_slice(&bits);
    }

    out
}

fn char_to_vec(c: u8) -> Vec<bool> {
    (0..7)
        .map(|i| (c >> i) & 1 == 1)
        .collect()
}

fn string_to_vec(s: &str) -> Vec<bool> {
    let mut v: Vec<bool> = Vec::new();

    for c in s.bytes() {
        v.extend(char_to_vec(c));
    }

    v.extend_from_slice(&[false; 7]);

    v
}

static LABEL_IDX: AtomicUsize = AtomicUsize::new(0);

fn decode_instr(line: &str) -> Vec<Instruction> {

    let line = line.replace(',', " ");
    let parts = line
        .split_whitespace()
        .collect::<Vec<&str>>();

    if parts[0] == "str" {
        let s = &line[5..line.len()-1];
        let s = s.replace("\\n", "\n");

        let v = string_to_vec(&s);
        return vec![Instruction::Data(v)];
    } else if parts[0] == "bits" {
        let b = &line[5..];

        let v: Vec<bool> = b.chars()
            .map(|c| match c {
                '0' => false,
                '1' => true,
                _ => panic!()
            })
            .collect();

        return vec![Instruction::Data(v)];
    }

    if line.ends_with(':') {
        let lab = line[..line.len()-1].to_string();
        return vec![Instruction::Label(lab)];
    }

    let op = parts[0];
    let args = parts[1..].iter()
        .map(|x| ValType::parse(x))
        .collect::<Vec<ValType>>();

    let tup = (op, &args[..]);

    match tup {
        ("add", &[ValType::Reg(ra), ValType::Reg(rb)]) => vec![Instruction::Add(ra, rb)],
        ("xor", &[ValType::Reg(ra), ValType::Reg(rb)]) => vec![Instruction::Xor(ra, rb)],
        ("and", &[ValType::Reg(ra), ValType::Reg(rb)]) => vec![Instruction::And(ra, rb)],
        ("or", &[ValType::Reg(ra), ValType::Reg(rb)]) => vec![Instruction::Or(ra, rb)],

        ("add", &[ValType::Reg(ra), ValType::Imm(imm)]) => vec![Instruction::AddI(ra, imm)],
        ("xor", &[ValType::Reg(ra), ValType::Imm(imm)]) => vec![Instruction::XorI(ra, imm)],
        ("and", &[ValType::Reg(ra), ValType::Imm(imm)]) => vec![Instruction::AndI(ra, imm)],
        ("or", &[ValType::Reg(ra), ValType::Imm(imm)]) => vec![Instruction::OrI(ra, imm)],

        ("load", &[ValType::Reg(ra), ValType::Reg(rb)]) => vec![Instruction::Load(ra, rb)],
        ("load", &[ValType::Reg(ra), ValType::Imm(imm)]) => vec![Instruction::LoadI(ra, imm)],
        ("load", &[ValType::Reg(ra), ValType::Label(lab)]) => vec![Instruction::LoadLabel(ra, lab.to_string())],

        ("store", &[ValType::Reg(ra), ValType::Reg(rb)]) => vec![Instruction::Store(ra, rb)],
        ("store", &[ValType::Reg(ra), ValType::Imm(imm)]) => vec![Instruction::StoreI(ra, imm)],
        ("store", &[ValType::Reg(ra), ValType::Label(lab)]) => vec![Instruction::StoreLabel(ra, lab.to_string())],

        ("jump", &[ValType::Imm(imm)]) => vec![Instruction::Jump(imm)],
        ("jump", &[ValType::Label(lab)]) => vec![Instruction::JumpLabel(lab.to_string())],
        
        ("jr", &[ValType::Reg(ra)]) => vec![Instruction::JumpR(ra)],

        ("jeq", &[ValType::Reg(ra), ValType::Reg(rb), ValType::Imm(imm)]) => vec![Instruction::Jeq(ra, rb, imm)],
        ("jeq", &[ValType::Reg(ra), ValType::Reg(rb), ValType::Label(lab)]) => vec![Instruction::JeqLabel(ra, rb, lab.to_string())],

        ("in", &[]) => vec![Instruction::Input],
        ("out", &[]) => vec![Instruction::Output],

        // Meta instructions
        ("li", &[ValType::Reg(ra), ValType::Imm(imm)]) => vec![
            Instruction::Xor(ra, ra),
            Instruction::AddI(ra, imm)
        ],

        ("li", &[ValType::Reg(ra), ValType::Label(lab)]) => vec![
            Instruction::Xor(ra, ra),
            Instruction::OrLabel(ra, lab.to_string())
        ],

        ("mov", &[ValType::Reg(ra), ValType::Reg(rb)]) => vec![
            Instruction::Xor(ra, ra),
            Instruction::Add(ra, rb)
        ],

        ("halt", &[]) => vec![
            Instruction::Xor(0, 0),
            Instruction::AddI(0, 0xff),
            Instruction::Output
        ],

        ("push", &[ValType::Reg(ra)]) => vec![
            Instruction::AddI(7, 0xfff0),
            Instruction::Store(ra, 7)
        ],

        ("pop", &[ValType::Reg(ra)]) => vec![
            Instruction::Load(ra, 7),
            Instruction::AddI(7, 0x10)
        ],

        ("call", &[ValType::Label(lab)]) => {
            let idx = LABEL_IDX.fetch_add(1, Ordering::SeqCst);
            let rlab = format!("tmp_{idx}");

            vec![
                // r0 = rlab
                Instruction::Xor(0,0),
                Instruction::OrLabel(0, rlab.clone()),

                // push r0
                Instruction::AddI(7, 0xfff0),
                Instruction::Store(0, 7),

                // jump func
                Instruction::JumpLabel(lab.to_string()),
                Instruction::Label(rlab)
            ]
        },

        ("ret", &[]) => vec![
            // pop r0
            Instruction::Load(0, 7),
            Instruction::AddI(7, 0x10),

            // jump r0
            Instruction::JumpR(0)
        ],

        _ => {
            panic!("Unknown instruction: {:?}", parts);
        }
    }
}

pub fn assemble(asm: &str) -> Vec<bool> {
    let instr = asm.split('\n')
        .map(|line| line.split(';').next().unwrap().trim())
        .filter(|line| line.len() > 0)
        .flat_map(decode_instr)
        .collect::<Vec<Instruction>>();

    println!("instr: {:#?}", instr);

    encode(&instr)
}
