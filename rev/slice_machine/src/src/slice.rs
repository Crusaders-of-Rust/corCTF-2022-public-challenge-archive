

use std::{rc::Rc, io::Read, collections::HashMap, sync::{atomic::{AtomicUsize, Ordering}, Arc, RwLock}, mem::ManuallyDrop, fmt::Debug, hash::Hash};

use crate::{context::Context, machine::UnsafeContext};

use indicatif::ProgressBar;
use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SliceRef {
    Rc(Rc<SliceExpr>),
    Idx(usize)
}

impl SliceRef {
    pub fn new_rc(expr: SliceExpr) -> Self {
        Self::Rc(Rc::new(expr))
    }

    pub fn eval(&self, ctx: &UnsafeContext, k: usize) -> Result<bool,()> {
        if let SliceRef::Rc(v) = self {
            v.eval(ctx, k)
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slice2 {
    pub a: SliceRef,
    pub b: SliceRef
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slice3 {
    pub c: SliceRef,
    pub t: SliceRef,
    pub f: SliceRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub r: usize
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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


#[derive(Debug, Serialize, Deserialize)]
pub struct SliceCache {
    pub slices: Vec<SliceExpr>,
    pub slice_ref: Vec<usize>,
}

impl SliceCache {
    pub fn from_vec(vec: &[SliceExpr]) -> Self {
        let mut cache = Self {
            slices: Vec::new(),
            slice_ref: Vec::new(),
        };

        let mut pack_cache: HashMap<usize, usize> = HashMap::new();

        for s in vec.iter() {
            let v = cache.pack(s, &mut pack_cache);
            cache.slice_ref.push(v);
        }

        cache
    }

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

    pub fn pack_ref(&mut self, r: &SliceRef, cache: &mut HashMap<usize,usize>) -> SliceRef {
        match r {
            SliceRef::Rc(v) => { 
                SliceRef::Idx(self.pack(v.as_ref(), cache))
            }
            SliceRef::Idx(_) => r.clone()
        }
    }

    pub fn pack(&mut self, s: &SliceExpr, cache: &mut HashMap<usize,usize>) -> usize {
        let ptr = (s as *const SliceExpr as *const usize as usize);
        
        if cache.contains_key(&ptr) {
            return *cache.get(&ptr).unwrap();
        }

        let s2 = match s {
            SliceExpr::Eq(Slice2 { a, b }) => {
                SliceExpr::Eq(Slice2 { a: self.pack_ref(a, cache), b: self.pack_ref(b, cache) })
            },
            SliceExpr::Cond(Slice3 { c, t, f }) => {
                SliceExpr::Cond(Slice3 { c: self.pack_ref(c, cache), t: self.pack_ref(t, cache), f: self.pack_ref(f, cache) })
            }
            SliceExpr::BitRef(_) => s.clone(),
            SliceExpr::Bit(_) => s.clone(),
            SliceExpr::Or(Slice2 { a, b }) => {
                SliceExpr::Or(Slice2 { a: self.pack_ref(a, cache), b: self.pack_ref(b, cache) })
            },
            SliceExpr::And(Slice2 { a, b }) => {
                SliceExpr::And(Slice2 { a: self.pack_ref(a, cache), b: self.pack_ref(b, cache) })
            },
            SliceExpr::Xor(Slice2 { a, b }) => {
                SliceExpr::Xor(Slice2 { a: self.pack_ref(a, cache), b: self.pack_ref(b, cache) })
            },
            SliceExpr::Input(_) => s.clone(),
            SliceExpr::Output(_,_) => s.clone(),
        };

        let idx = self.slices.len();
        cache.insert(ptr, idx);
        self.slices.push(s2);

        idx
    }
}
