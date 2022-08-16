
use std::cell::{UnsafeCell};
use std::fs::File;
use std::io::{Seek, SeekFrom, Read};

use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::{Serialize, Deserialize};

use crate::context::Context;
use crate::slice::*;


const TEMPORAL: usize = 1000;

#[derive(Serialize, Deserialize)]
pub struct MachineFile {
    pub initial: Vec<bool>,
    pub cache: SliceCache,
}

pub struct Machine {
    pub initial: Vec<bool>,
    pub slices: Vec<SliceExpr>,
}

unsafe impl Sync for Machine {}

pub struct UnsafeContext {
    ctx: UnsafeCell<Context>
}

impl UnsafeContext {
    pub fn new(ctx: Context) -> Self {
        UnsafeContext { ctx: UnsafeCell::new(ctx) }
    }

    pub fn get_mut(&self) -> &mut Context {
        let x = self as *const UnsafeContext as *mut UnsafeContext;
        unsafe {
            x.as_mut()
        }.unwrap().ctx.get_mut()
    }
}

unsafe impl Sync for UnsafeContext {}

impl Machine {
    pub fn load(filepath: &str) -> Self {
        println!("[*] Loading: [{filepath}]...",);

        let mut f = File::open(filepath).unwrap();

        f.seek(SeekFrom::End(0)).unwrap();
        let size = f.stream_position().unwrap() as usize;
        f.seek(SeekFrom::Start(0)).unwrap();

        let mut buf: Vec<u8> = Vec::with_capacity(size);
        buf.resize(size, 0);
        f.read(&mut buf).unwrap();

        let file: MachineFile = rmp_serde::from_slice(&buf).unwrap();

        let slices = file.cache.to_vec();

        Machine {
            initial: file.initial.clone(),
            slices
        }
    }

    pub fn run(&self) {
        println!("[*] Running...");
        println!("--------------------");

        let mut rng = thread_rng();

        let mut ctx = Context::new_sized(self.slices.len(), TEMPORAL);
        for i in 0..self.initial.len() {
            ctx.set(i, 0, self.initial[i])
        }

        let cref = UnsafeContext::new(ctx);


        let mut t: usize = 0;
        loop {
            let mut unfilled = cref.get_mut().unfilled(t+1);
            unfilled.shuffle(&mut rng);

            let _ = unfilled
                .par_iter()
                .map(|idx| {
                    for tm in 0..TEMPORAL {
                        if let Ok(v) = self.slices[*idx].eval(&cref, t+tm) {
                            cref.get_mut().set(*idx, t+tm+1, v);
                        } else {
                            break;
                        }
                    }
                    ()
                })
                .collect::<Vec<()>>();

            if cref.get_mut().shift() {
                break;
            }
            t += 1;
        }

        println!("--------------------");
        println!("[*] Stopped");
    }
}
