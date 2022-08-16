
mod slice;
mod context;
mod machine;
mod assembler;

use assembler::assemble;
use slice::*;
use machine::Machine;

use crate::context::Context;

use std::{rc::Rc, io::Write};


macro_rules! cond {
    ($c:expr, $t:expr, $f:expr) => {
        SliceExpr::Cond(Slice3 {
            c: SliceRef::new_rc($c),
            t: SliceRef::new_rc($t),
            f: SliceRef::new_rc($f),
        })
    };
}

macro_rules! eq {
    ($a:expr, $b:expr) => {
        SliceExpr::Eq(Slice2 {
            a: SliceRef::new_rc($a),
            b: SliceRef::new_rc($b),
        })
    };
}

macro_rules! or {
    ($a:expr, $b:expr) => {
        SliceExpr::Or(Slice2 {
            a: SliceRef::new_rc($a),
            b: SliceRef::new_rc($b),
        })
    };
}

macro_rules! and {
    ($a:expr, $b:expr) => {
        SliceExpr::And(Slice2 {
            a: SliceRef::new_rc($a),
            b: SliceRef::new_rc($b),
        })
    };
}

macro_rules! xor {
    ($a:expr, $b:expr) => {
        SliceExpr::Xor(Slice2 {
            a: SliceRef::new_rc($a),
            b: SliceRef::new_rc($b),
        })
    };
}

macro_rules! bitref {
    ($r:expr) => {
        SliceExpr::BitRef(
            Index {
                r: $r,
            }
        )
    }
}

macro_rules! input {
    ($r:expr) => {
        SliceExpr::Input(
            Index {
                r: $r,
            }
        )
    }
}

macro_rules! output {
    ($r:expr, $v:expr) => {
        SliceExpr::Output(
            Index {
                r: $r,
            },
            Value {
                v: $v
            }
        )
    }
}

macro_rules! bit {
    ($v:expr) => {
        SliceExpr::Bit(
            Value {
                v: $v,
            }
        )
    }
}

fn vand(a: &[SliceExpr], b: &[SliceExpr], N: usize) -> Vec<SliceExpr> {
    (0..N).map(|n| and!(a[n].clone(), b[n].clone()))
        .collect()
}

fn vor(a: &[SliceExpr], b: &[SliceExpr], N: usize) -> Vec<SliceExpr> {
    (0..N).map(|n| or!(a[n].clone(), b[n].clone()))
        .collect()
}

fn vxor(a: &[SliceExpr], b: &[SliceExpr], N: usize) -> Vec<SliceExpr> {
    (0..N).map(|n| xor!(a[n].clone(), b[n].clone()))
        .collect()
}

/// Imm of size N
fn imm(v: usize, N: usize) -> Vec<SliceExpr> {
    (0..N).map(|n| bit!((v >> n) & 1 == 1))
        .collect::<Vec<SliceExpr>>()
}

/// Absolute slice of bits at offset p of size N.
fn slice(p: usize, N: usize) -> Vec<SliceExpr> {
    (0..N).map(|n| bitref!(p+n))
        .collect::<Vec<SliceExpr>>()
}

fn strided(mem: &[SliceExpr], N: usize, size: usize, stride: usize) -> Vec<Vec<SliceExpr>> {
    (0..size).map(|s| {
        (0..N).map(|n| mem[(s + (n * stride)) as usize].clone())
            .collect::<Vec<SliceExpr>>()
    }).collect::<Vec<Vec<SliceExpr>>>()
}

fn ext_input() -> Vec<SliceExpr> {
    vec![
        input!(0),
        input!(1),
        input!(2),
        input!(3),
        input!(4),
        input!(5),
        input!(6),
        input!(7),
        bit!(false),
        bit!(false),
        bit!(false),
        bit!(false),
        bit!(false),
        bit!(false),
        bit!(false),
        bit!(false),
    ]
}

/// Add two ref slices of size N with a full-adder circuit.
fn add(a: &[SliceExpr], b: &[SliceExpr], N: usize) -> Vec<SliceExpr> {
    let mut c = Vec::new();
    let mut s = Vec::new();

    for i in 0..N {
        if i == 0 {
            s.push(xor!(a[i].clone(), b[i].clone()));
            c.push(and!(a[i].clone(), b[i].clone()));
        } else {
            // s = (a xor b) xor cin
            s.push(
                xor!(
                    xor!(a[i].clone(), b[i].clone()),
                    c[i-1].clone()
                )
            );
            // cout = ((a xor b) and cin) or (a and b)
            c.push(
                or!(
                    and!(
                        xor!(a[i].clone(), b[i].clone()),
                        c[i-1].clone()
                    ),
                    and!(a[i].clone(), b[i].clone())
                )
            );
        }
    }

    s
}

/// Vector equality.
fn veq(a: &[SliceExpr], b: &[SliceExpr], N: usize) -> SliceExpr {
    let mut res = eq!(a[0].clone(), b[0].clone());

    for i in 1..N {
        res = and!(res, eq!(a[i].clone(), b[i].clone()));
    }

    res
}

/// Multiplex a bit ref slice m with a selector s of size N.
/// Size of m should be 2^N.
fn bmux(m: &[SliceExpr], N: u32, v: &[SliceExpr]) -> SliceExpr {
    // println!("bmux {:?}", N);
    match N {
        1 => { cond!(m[0].clone(), v[1].clone(), v[0].clone()) }
        _ => { 
            cond!(
                m[(N as usize)-1].clone(),
                bmux(m, N-1, &v[(2usize).pow(N-1)..]),
                bmux(m, N-1, &v[..(2usize).pow(N-1)])
            )
        }
    }
}

fn mux(sel: &[SliceExpr], N: u32, mem: &[SliceExpr], size: usize, stride: usize) -> Vec<SliceExpr> {
    strided(mem, (2usize).pow(N), size, stride).iter()
        .map(|st| bmux(&sel, N, &st))
        .collect::<Vec<SliceExpr>>()
}

// Evaluate a vec for testing.
macro_rules! eval {
    ($v:expr, $ctx:expr, $k:expr) => {
        $v.iter()
            .map(|x| x.eval(&$ctx, $k))
            .collect::<Vec<Result<bool,()>>>()
    };
}

fn as_num(r: &[Result<bool, ()>]) -> Option<usize> {
    if r.iter().all(|x| x.is_ok()) {
        Some(r.iter()
            .filter_map(|x| x.ok().map(|v| v as usize))
            .rfold(0, |acc, x| (acc << 1) | x))
    } else {
        None
    }
}

fn vcond(c: SliceExpr, a: &[SliceExpr], b: &[SliceExpr], N: usize) -> Vec<SliceExpr> {
    (0..N).map(|n| cond!(c.clone(), a[n].clone(), b[n].clone()))
        .collect::<Vec<SliceExpr>>()
}

// Bit offsets:
const ROM_BITS: usize = 13;
const MEM_BITS: usize = 14;
const ROM: usize = 0;
const ROM_SIZE: usize = 0x2000;
const RAM: usize = 0x2000;
const RAM_SIZE: usize = 0x2000;
const PC: usize = 0x4000;
const REGS: usize = 0x4010;
const REG_SIZE: usize = 16;

// Encodings:
// R-type: [op:4][ra:3][rb:3]          -- 10
// I-type: [op:4][ra:3][imm:16]        -- 23
// B-type: [op:4][ra:3][rb:3][imm:16]  -- 26
// Z-type: [op:4][d:1]                 -- 5
const RTYPE_RA: usize = 4;
const RTYPE_RB: usize = 7;

const ITYPE_RA: usize = 4;
const ITYPE_IMM: usize = 7;

const BTYPE_RA: usize = 4;
const BTYPE_RB: usize = 7;
const BTYPE_IMM: usize = 10;

const ZTYPE_D: usize = 4;


macro_rules! rom {
    () => { slice(ROM, ROM_SIZE) };
}

macro_rules! pc {
    () => { slice(PC, 16) };
}

macro_rules! reg {
    ($r:expr) => {
        slice(REGS + ($r * REG_SIZE), REG_SIZE)
    };
}

// Testing
macro_rules! set_pc {
    ($v:expr, $ctx:expr, $k:expr) => {
        for i in 0..16 {
            $ctx.get_mut().set(PC + i, $k, ($v >> i) & 1 == 1);
        }
    };
}

macro_rules! set_reg {
    ($r:expr, $v:expr, $ctx:expr, $k:expr) => {
        for i in 0..16 {
            $ctx.get_mut().set(REGS + ($r * REG_SIZE) + i, $k, ($v >> i) & 1 == 1);
        }
    };
}

macro_rules! write_imm {
    ($p:expr, $v:expr, $N:expr, $ctx:expr, $k:expr) => {
        for i in 0..$N {
            $ctx.get_mut().set($p + i, $k, ($v >> i) & 1 == 1);
        }
    };
}

macro_rules! write {
    ($p:expr, $v:expr, $ctx:expr, $k:expr) => {
        for i in 0..$v.len() {
            $ctx.get_mut().set($p + i, $k, $v[i] == 1);
        }
    };
}

fn get_reg(sel: &[SliceExpr]) -> Vec<SliceExpr> {
    mux(sel, 3, &slice(REGS, REG_SIZE * 8), REG_SIZE, REG_SIZE)
}

fn get_field(off: usize, size: usize) -> Vec<SliceExpr> {
    let sel = {
        if off > 0 {
            add(&pc!()[..ROM_BITS], &imm(off, ROM_BITS), ROM_BITS)
        } else {
            pc!()[..ROM_BITS].to_vec()
        }
    };
    mux(&sel, ROM_BITS as u32, &slice(ROM, ROM_SIZE+size), size, 1)
}

fn build_pc() -> Vec<SliceExpr> {
    let op = get_field(0, 4);
    let a = &op[0];
    let b = &op[1];
    let c = &op[2];
    let d = &op[3];

    let add_5 = add(&pc!(), &imm(5, REG_SIZE), REG_SIZE);
    let add_10 = add(&pc!(), &imm(10, REG_SIZE), REG_SIZE);
    let add_23 = add(&pc!(), &imm(23, REG_SIZE), REG_SIZE);
    let add_26 = add(&pc!(), &imm(26, REG_SIZE), REG_SIZE);

    let ra = get_reg(&get_field(BTYPE_RA, 3));
    let rb = get_reg(&get_field(BTYPE_RB, 3));

    let b_imm = get_field(BTYPE_IMM, 16);

    vcond(
        eq!(a.clone(), bit!(false)),
        &vcond(
            eq!(b.clone(), bit!(false)),
            &add_10, // R-type [00xx]
            &add_23, // I-type [01xx]
            REG_SIZE
        ),
        &vcond(
            eq!(b.clone(), bit!(false)),
            &vcond( // [10xx]
                eq!(c.clone(), bit!(false)),
                &add_10, // R-type [100x]
                &add_23, // I-type [101x]
                REG_SIZE
            ),
            &vcond( // [11xx]
                eq!(c.clone(), bit!(false)),
                &vcond( // [110x]
                    eq!(d.clone(), bit!(false)),
                    &b_imm, // [1100] j imm
                    &get_reg(&get_field(BTYPE_RA, 16)), // [1101] j ra
                    REG_SIZE
                ),
                &vcond( // [111x]
                    eq!(d.clone(), bit!(false)),
                    &vcond( // [1110] jeq ra, rb, imm
                        veq(&ra, &rb, REG_SIZE),
                        &b_imm,
                        &add_26,
                        REG_SIZE
                    ),
                    &add_5, // Z-type [1111]
                    REG_SIZE
                ),
                REG_SIZE
            ),
            REG_SIZE
        ),
        REG_SIZE
    )
}

fn build_rom(idx: usize) -> SliceExpr {
    bitref!(idx)
}

fn build_reg(ridx: usize) -> Vec<SliceExpr> {
    // - R [0000]: add ra, rb
    // - R [0001]: xor ra, rb
    // - R [0010]: and ra, rb
    // - R [0011]: or ra, rb

    // - I [0100]: add ra, imm
    // - I [0101]: xor ra, imm
    // - I [0110]: and ra, imm
    // - I [0111]: or ra, imm

    // - R [1000]: ld ra, [rb]
    // X R [1001]: st ra, [rb]
    // - I [1010]: ld ra, [imm]
    // X I [1011]: st ra, [imm]

    // X [11xx]

    // X B [1100]: j imm
    // X B [1101]: j ra
    // X B [1110]: jeq ra, rb, imm
    // X Z [1111]:
    //     - (d==0): write r0
    //     - (d==1): read r0

    let curr = reg!(ridx);

    let op = get_field(0, 4);
    let a = &op[0];
    let b = &op[1];
    let c = &op[2];
    let d = &op[3];

    let ra_idx = get_field(RTYPE_RA, 3);
    let rb_idx = get_field(RTYPE_RB, 3);

    let ra = get_reg(&ra_idx);
    let rb = get_reg(&rb_idx);
    let instr_imm = get_field(ITYPE_IMM, 16);
    let io_dir = &get_field(ZTYPE_D, 1)[0];

    let process_rtype = vcond(
        eq!(c.clone(), bit!(false)),
        &vcond(
            eq!(d.clone(), bit!(false)),
            &add(&ra, &rb, REG_SIZE),
            &vxor(&ra, &rb, REG_SIZE),
            REG_SIZE
        ),
        &vcond(
            eq!(d.clone(), bit!(false)),
            &vand(&ra, &rb, REG_SIZE),
            &vor(&ra, &rb, REG_SIZE),
            REG_SIZE
        ),
        REG_SIZE
    );

    let ld_ra_rb = mux(
        &rb[..MEM_BITS],
        MEM_BITS as u32,
        &slice(ROM, ROM_SIZE + RAM_SIZE + REG_SIZE),
        REG_SIZE,
        1
    );

    let ld_ra_imm = mux(
        &instr_imm[..MEM_BITS],
        MEM_BITS as u32,
        &slice(ROM, ROM_SIZE + RAM_SIZE + REG_SIZE),
        REG_SIZE,
        1
    );

    let process_itype = vcond(
        eq!(c.clone(), bit!(false)),
        &vcond(
            eq!(d.clone(), bit!(false)),
            &add(&ra, &instr_imm, REG_SIZE),
            &vxor(&ra, &instr_imm, REG_SIZE),
            REG_SIZE
        ),
        &vcond(
            eq!(d.clone(), bit!(false)),
            &vand(&ra, &instr_imm, REG_SIZE),
            &vor(&ra, &instr_imm, REG_SIZE),
            REG_SIZE
        ),
        REG_SIZE
    );

    let process_load_store = vcond(
        eq!(c.clone(), bit!(false)),
        &vcond(
            eq!(d.clone(), bit!(false)),
            &ld_ra_rb,
            &curr,
            REG_SIZE
        ),
        &vcond(
            eq!(d.clone(), bit!(false)),
            &ld_ra_imm,
            &curr,
            REG_SIZE
        ),
        REG_SIZE
    );

    let process_other = {
        vcond(
            and!(
                and!(
                    eq!(c.clone(), bit!(true)),
                    eq!(d.clone(), bit!(true))
                ),
                eq!(io_dir.clone(), bit!(true))
            ),
            &ext_input(),
            &curr.clone(),
            REG_SIZE
        )
    };

    let is_io = veq(&op, &imm(0b1111, 4), 4);

    let exec_cond = if ridx == 0 {
        or!(
            is_io.clone(),
            veq(&ra_idx, &imm(ridx, 3), 3)
        )
    } else {
        and!(
            cond!(
                is_io.clone(),
                bit!(false),
                bit!(true)
            ),
            veq(&ra_idx, &imm(ridx, 3), 3)
        )
    };

    vcond(
        exec_cond,
        &vcond(
            eq!(a.clone(), bit!(false)),
            &vcond(
                eq!(b.clone(), bit!(false)),
                &process_rtype, // [00xx]
                &process_itype, // [01xx]
                REG_SIZE
            ),
            &vcond(
                eq!(b.clone(), bit!(false)),
                &process_load_store, // [10xx]
                &process_other, // [11xx]
                REG_SIZE
            ),
            REG_SIZE
        ),
        &curr,
        REG_SIZE
    )
}

fn build_ram(start: usize, end: usize) -> Vec<SliceExpr> {
    // X R [00xx]
    // X I [01xx]

    // X R [1000]: ld ra, [rb]
    // - R [1001]: st ra, [rb]
    // X I [1010]: ld ra, [imm]
    // - I [1011]: st ra, [imm]

    // X B [11xx]

    let op = get_field(0, 4);
    let a = &op[0];
    let b = &op[1];
    let c = &op[2];
    let d = &op[3];

    let ra_idx = get_field(RTYPE_RA, 3);
    let rb_idx = get_field(RTYPE_RB, 3);

    let ra = get_reg(&ra_idx);
    let rb = get_reg(&rb_idx);
    let instr_imm = get_field(ITYPE_IMM, 16);

    let addr = vcond(
        eq!(c.clone(), bit!(false)),
        &rb,
        &instr_imm,
        REG_SIZE
    );

    let mut ram: Vec<SliceExpr> = Vec::with_capacity(end - start);

    for idx in start..end {
        let curr = bitref!(idx);

        let mut comp = cond!(
            veq(&addr, &imm(idx, REG_SIZE), REG_SIZE),
            ra[0].clone(),
            curr.clone()
        );

        for i in 1..16 {
            comp = cond!(
                veq(&addr, &imm(idx-i, REG_SIZE), REG_SIZE),
                ra[i].clone(),
                comp
            );
        }

        let process_load_store = cond!(
            eq!(d.clone(), bit!(true)),
            comp,
            curr.clone() // [1000, 1010]
        );

        let b = cond!(
            eq!(a.clone(), bit!(false)),
            curr.clone(),
            cond!(
                eq!(b.clone(), bit!(false)),
                process_load_store, // [10xx]
                curr.clone() // [11xx]
            )
        );

        ram.push(b);
    }

    ram
}

fn build_output() -> Vec<SliceExpr> {
    let op = get_field(0, 4);
    let io_dir = &get_field(ZTYPE_D, 1)[0];
    let r0 = reg!(0);

    let mut v = Vec::new();

    for i in 0..8 {
        let p = cond!(
            and!(
                veq(&op, &imm(0b1111, 4), 4),
                eq!(io_dir.clone(), bit!(false))
            ),
            cond!(
                r0[i].clone(),
                output!(i, true),
                output!(i, false)
            ),
            bit!(false)
        );

        v.push(p);
    }

    v
}

fn build_machine() -> Machine {
    let sz = 0x4000 + (REG_SIZE * 9) + 8;
    let mut slices = Vec::with_capacity(sz);
    slices.resize(sz, bit!(false));

    for i in 0..0x2000 {
        slices[i] = build_rom(i);
    }

    let ram = build_ram(0x2000, 0x4000);
    for i in 0x2000..0x4000 {
        slices[i] = ram[i-0x2000].clone();
    }

    let pc = build_pc();
    for i in 0..16 {
        slices[0x4000+i] = pc[i].clone();
    }

    for r in 0..8 {
        let reg = build_reg(r);
        for i in 0..16 {
            slices[0x4010 + (r * 16) + i] = reg[i].clone();
        }
    }

    let out = build_output();
    for i in 0..8 {
        slices[0x4090 + i] = out[i].clone();
    }

    let initial = vec![false; sz];

    Machine {
        initial,
        slices
    }
}


fn main() {

    let s = include_str!("./prog.txt");
    let out = assemble(s);

    println!("out: {:?}", out);
    println!("size: 0x{:x}", out.len());

    let mut m = build_machine();
    for i in 0..out.len() {
        m.initial[i] = out[i];
    }

    m.shuffle(19749283647);

    m.save("./machine.dat");

    // let m = Machine::load("./machine.dat");
    // m.run();
}


#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use crate::{*, machine::UnsafeContext};

    #[test]
    fn test_store() {
        let ctx = UnsafeContext::new(Context::new());

        let bits = &[
            1,0,0,1, 1,0,0, 0,1,0, // st r1, [r2]
        ];
        for i in 0..3 {
            write!(0, bits, ctx, i);
        }
        set_pc!(0, ctx, 0);
        set_reg!(1, 123, ctx, 0); // r1 = 123
        set_reg!(2, 40, ctx, 0); // r2 = 40

        let ram: Vec<SliceExpr> = build_ram(30, 60);

        let res = eval!(ram, ctx, 0);

        assert_eq!(
            res,
            &[Err(()), Err(()), Err(()), Err(()), Err(()), Err(()), Err(()), Err(()), Err(()), Err(()), 
              Ok(true), Ok(true), Ok(false), Ok(true), Ok(true), Ok(true), Ok(true), Ok(false), Ok(false), Ok(false), 
              Ok(false), Ok(false), Ok(false), Ok(false), Ok(false), Ok(false), Err(()), Err(()), Err(()), Err(())]
        );
    }

    #[test]
    fn test_multi() {
        let ctx = UnsafeContext::new(Context::new());

        let bits = &[
            1,0,0,0,  1,0,0,  0,1,0, // ld r1, [r2]
            0,0,0,0,  0,1,0,  0,1,0, // add r2, r2
            1,0,0,0,  1,0,0,  0,1,0, // ld r1, [r2]
        ];
        for i in 0..3 {
            write!(0, bits, ctx, i);
            write_imm!(50, 12345, 16, ctx, i);
            write_imm!(100, 34567, 16, ctx, i);
        }

        set_pc!(0, ctx, 0);
        set_reg!(1, 123, ctx, 0); // r1 = 123
        set_reg!(2, 50, ctx, 0); // r2 = 50

        let slice_pc = build_pc();
        let reg1 = build_reg(1);
        let reg2 = build_reg(2);

        let expected = &[
            &[Some(10), Some(12345), Some(50)],
            &[Some(20), Some(12345), Some(100)],
            &[Some(30), Some(34567), Some(100)],
        ];

        for i in 0..3 {
            let next_pc = as_num(&eval!(slice_pc, ctx, i));
            let next_r1 = as_num(&eval!(reg1, ctx, i));
            let next_r2 = as_num(&eval!(reg2, ctx, i));

            println!("pc: {:?}", next_pc);
            println!("r1: {:?}", next_r1);
            println!("r2: {:?}", next_r2);

            assert_eq!(
                &[next_pc, next_r1, next_r2],
                expected[i]
            );

            set_pc!(next_pc.unwrap(), ctx, i+1);
            set_reg!(1, next_r1.unwrap(), ctx, i+1);
            set_reg!(2, next_r2.unwrap(), ctx, i+1);
        }
    }

    #[test]
    fn test_get_op() {
        let ctx = UnsafeContext::new(Context::new());

        let bits = &[0,1,1,0,0,0,1,0,0,1,1,0,0,1,0,1];
        for i in 0..bits.len() {
            ctx.get_mut().set(i, 0, bits[i] == 1);
        }

        let op = get_field(0, 4);

        for i in 0..13 {
            // Set PC value
            for j in 0..16 {
                ctx.get_mut().set(PC + j, 0, (i >> j) & 1 == 1);
            }
            
            let res = eval!(op, ctx, 0);
            let num = as_num(&res);

            let exp = &bits[i..i+4];
            let n = exp.iter().rfold(0, |acc, x| (acc << 1) | x);
            
            assert_eq!(num, Some(n));
        }
    }

    #[test]
    fn test_adder_imm() {
        let ctx = UnsafeContext::new(Context::new());

        // 12
        ctx.get_mut().set(0, 0, false);
        ctx.get_mut().set(1, 0, false);
        ctx.get_mut().set(2, 0, true);
        ctx.get_mut().set(3, 0, true);

        let a = slice(0,4);

        for i in 0..16 {
            let b = imm(i,4);
            let c = add(&a, &b, 4);

            let r = eval!(c, ctx, 0);
            let n = as_num(&r);

            assert_eq!(n, Some((12 + i) % 16))
        }
    }

    #[test]
    fn test_veq() {
        let ctx = UnsafeContext::new(Context::new());

        ctx.get_mut().set(0, 0, false);
        ctx.get_mut().set(1, 0, false);
        ctx.get_mut().set(2, 0, true);
        ctx.get_mut().set(3, 0, true);

        ctx.get_mut().set(4, 0, false);
        ctx.get_mut().set(5, 0, true);
        ctx.get_mut().set(6, 0, false);
        ctx.get_mut().set(7, 0, true);

        ctx.get_mut().set(8, 0, false);
        ctx.get_mut().set(9, 0, false);
        ctx.get_mut().set(10, 0, true);
        ctx.get_mut().set(11, 0, true);

        let a = slice(0,4);
        let b = slice(4,4);
        let c = slice(8,4);

        let r1 = veq(&a,&b,4).eval(&ctx,0);
        let r2 = veq(&a,&c,4).eval(&ctx,0);

        assert_eq!(r1, Ok(false));
        assert_eq!(r2, Ok(true));
    }

    #[test]
    fn multiplex_vec() {
        let ctx = UnsafeContext::new(Context::new());

        let base = 10;
        for i in 0..16 {
            let v = 10 + i;

            // 5 bit values
            for j in 0..5 {
                ctx.get_mut().set(base + (i * 5) + j, 0, (v >> j) & 1 == 1);
            }
        }

        let r = mux(&slice(0, 4), 4, &slice(base, 16 * 5), 5, 5);

        for k in 0..16 {
            let sel = k;
            for i in 0..4 {
                ctx.get_mut().set(i, 0, (sel >> i) & 1 == 1);
            }

            let z = eval!(r, ctx, 0);
            let num = as_num(&z);

            assert_eq!(num, Some(k+10));
        }
    }

    #[test]
    fn multiplex() {
        let ctx = UnsafeContext::new(Context::new());

        // 5
        ctx.get_mut().set(0, 0, true);
        ctx.get_mut().set(1, 0, false);
        ctx.get_mut().set(2, 0, true);
        
        ctx.get_mut().set(3, 0, false);
        ctx.get_mut().set(4, 0, false);
        ctx.get_mut().set(5, 0, false);
        ctx.get_mut().set(6, 0, false);
        ctx.get_mut().set(7, 0, false);
        ctx.get_mut().set(8, 0, true); // idx 5
        ctx.get_mut().set(9, 0, false);
        ctx.get_mut().set(10, 0, false);

        let m = slice(0, 3);
        let v = slice(3, 8);

        let r = bmux(&m, 3, &v);
        let v = r.eval(&ctx, 0);

        assert_eq!(v, Ok(true));
    }

    #[test]
    fn add_slices() {
        let ctx = UnsafeContext::new(Context::new());

        // 5
        ctx.get_mut().set(0, 0, true);
        ctx.get_mut().set(1, 0, false);
        ctx.get_mut().set(2, 0, true);
        ctx.get_mut().set(3, 0, false);

        // 6
        ctx.get_mut().set(4, 0, false);
        ctx.get_mut().set(5, 0, true);
        ctx.get_mut().set(6, 0, true);
        ctx.get_mut().set(7, 0, false);

        let A: usize = 0;
        let B: usize = 4;

        let a = slice(A, 4);
        let b = slice(B, 4);

        let c = add(&a,&b,4);

        let res = eval!(c, ctx, 0);
        let num = as_num(&res);

        assert_eq!(num, Some(11));
    }
}


/*
    VM Architecture

    ROM: [0 - 0x2000]
    RAM: [0x2000 - 0x4000]

    REG: [0x4000 - 0x4000]

    Regs:
    - PC: 16 bits
    - R0-R7: 16 bits

    Instructions:
    R-type: [op:4][ra:3][rb:3]          -- 10
    I-type: [op:4][ra:3][imm:16]        -- 23
    B-type: [op:4][ra:3][rb:3][imm:16]  -- 26
    Z-type: [op:4][d:1]                 -- 5

    - R [0000]: add ra, rb
    - R [0001]: xor ra, rb
    - R [0010]: and ra, rb
    - R [0011]: or ra, rb

    - I [0100]: add ra, imm
    - I [0101]: xor ra, imm
    - I [0110]: and ra, imm
    - I [0111]: or ra, imm

    - R [1000]: ld ra, [rb]
    - R [1001]: st ra, [rb]
    - I [1010]: ld ra, [imm]
    - I [1011]: st ra, [imm]

    - B [1100]: j imm
    - B [1101]: j ra
    - B [1110]: jeq ra, rb, imm

    - Z [1111]:
        - (d==0): write r0
        - (d==1): read r0

    
    PC: [abcd]
        (ab == 00) -> pc + 10
        (ab == 01) -> pc + 23
        (ab == 10)
            (c ? pc + 23 : pc + 10)
        ()
            ...

*/

/*

    Program that simulates a turing machine that checks the flag.


*/



/*

Slice:

b_k^{i+1} = f(b_k^{i})

Slices can apply short-circuiting protections to reduce computation.

E.g. memory bit can short circuit if the instruction 

*/

