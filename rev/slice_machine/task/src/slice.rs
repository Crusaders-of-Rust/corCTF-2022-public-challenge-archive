
use std::{rc::Rc, collections::HashMap};

use indicatif::ProgressBar;
use serde::{Serialize, Deserialize};

use crate::machine::UnsafeContext;

#[derive(Clone, Serialize, Deserialize)]
pub enum SliceRef {
    Rc(Rc<SliceExpr>),
    Idx(usize)
}

impl SliceRef {
    pub fn eval(&self, ctx: &UnsafeContext, k: usize) -> Result<bool,()> {
        if let SliceRef::Rc(v) = self {
            v.eval(ctx, k)
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum SliceExpr {
    Eq(Slice2),
    Cond(Slice3),
    BitRef(Index),
    Bit(Value),
    Or(Slice2),
    And(Slice2),
    Xor(Slice2),
    Input(Index),
    Output(Index, Value),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Slice2 {
    pub a: SliceRef,
    pub b: SliceRef
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Slice3 {
    pub c: SliceRef,
    pub t: SliceRef,
    pub f: SliceRef,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Index {
    pub r: usize
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Value {
    pub v: bool
}

impl SliceExpr {
    pub fn eval(&self, ctx: &UnsafeContext, k: usize) -> Result<bool,()> {
        Ok(match self {
            SliceExpr::Eq(Slice2 { a, b }) => {
                a.eval(ctx, k)? == b.eval(ctx, k)?
            }
            SliceExpr::BitRef(Index { r }) => {
                ctx.get_mut().get(*r, k).ok_or(())?
            }
            SliceExpr::Bit(Value { v }) => {
                *v
            }
            SliceExpr::Cond(Slice3 { c, t, f }) => {
                if c.eval(ctx, k)? {
                    t.eval(ctx, k)?
                } else {
                    f.eval(ctx, k)?
                }
            }
            SliceExpr::Or(Slice2 { a, b }) => {
                a.eval(ctx, k)? || b.eval(ctx, k)?
            }
            SliceExpr::And(Slice2 { a, b }) => {
                a.eval(ctx, k)? && b.eval(ctx, k)?
            }
            SliceExpr::Xor(Slice2 { a, b }) => {
                a.eval(ctx, k)? ^ b.eval(ctx, k)?
            }
            SliceExpr::Input(Index { r }) => {
                ctx.get_mut().input(*r, k).ok_or(())?
            }
            SliceExpr::Output(Index {r}, Value {v}) => {
                ctx.get_mut().output(*r, *v, k);
                *v
            }
        })
    }
}


#[derive(Serialize, Deserialize)]
pub struct SliceCache {
    pub slices: Vec<SliceExpr>,
    pub slice_ref: Vec<usize>,
}

impl SliceCache {
    pub fn to_vec(&self) -> Vec<SliceExpr> {
        let mut out = Vec::with_capacity(self.slice_ref.len());

        let mut unpack_cache: HashMap<usize, Rc<SliceExpr>> = HashMap::new();

        let bar = ProgressBar::new(self.slice_ref.len() as u64);

        for idx in self.slice_ref.iter() {
            let v = self.unpack(*idx, &mut unpack_cache);
            out.push(v.as_ref().clone());

            bar.inc(1);
        }
        bar.finish();

        out
    }

    pub fn unpack_ref(&self, r: &SliceRef, cache: &mut HashMap<usize, Rc<SliceExpr>>) -> SliceRef {
        match r {
            SliceRef::Rc(_) => r.clone(),
            SliceRef::Idx(idx) => {
                SliceRef::Rc(self.unpack(*idx, cache))
            }
        }
    }

    pub fn unpack(&self, idx: usize, cache: &mut HashMap<usize, Rc<SliceExpr>>) -> Rc<SliceExpr> {
        if let Some(v) = cache.get(&idx) {
            return v.clone();
        }

        let s = &self.slices[idx];

        let s2 = match s {
            SliceExpr::Eq(Slice2 { a, b }) => {
                SliceExpr::Eq(Slice2 { a: self.unpack_ref(a, cache), b: self.unpack_ref(b, cache) })
            },
            SliceExpr::Cond(Slice3 { c, t, f }) => {
                SliceExpr::Cond(Slice3 { c: self.unpack_ref(c, cache), t: self.unpack_ref(t, cache), f: self.unpack_ref(f, cache) })
            }
            SliceExpr::BitRef(_) => s.clone(),
            SliceExpr::Bit(_) => s.clone(),
            SliceExpr::Or(Slice2 { a, b }) => {
                SliceExpr::Or(Slice2 { a: self.unpack_ref(a, cache), b: self.unpack_ref(b, cache) })
            },
            SliceExpr::And(Slice2 { a, b }) => {
                SliceExpr::And(Slice2 { a: self.unpack_ref(a, cache), b: self.unpack_ref(b, cache) })
            },
            SliceExpr::Xor(Slice2 { a, b }) => {
                SliceExpr::Xor(Slice2 { a: self.unpack_ref(a, cache), b: self.unpack_ref(b, cache) })
            },
            SliceExpr::Input(_) => s.clone(),
            SliceExpr::Output(_,_) => s.clone(),
        };

        let v = Rc::new(s2);
        cache.insert(idx, v.clone());
        v
    }
}
