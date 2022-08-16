
use core::panic;
use std::cell::{UnsafeCell};
use std::fs::File;
use std::io::{Write, Seek, SeekFrom, Read};
use std::ops::DerefMut;
use std::os::unix::thread;
use std::sync::{Arc, Mutex, RwLock};

use indicatif::ProgressBar;
use rand::rngs::StdRng;
use rand::seq::{SliceRandom, index};
use rand::{thread_rng, SeedableRng};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::{Serialize, Deserialize};

use crate::*;
use crate::slice::*;


const TEMPORAL: usize = 32;

#[derive(Debug, Serialize, Deserialize)]
pub struct MachineFile {
    pub initial: Vec<bool>,
    pub cache: SliceCache,
}

#[derive(Debug)]
pub struct Machine {
    pub initial: Vec<bool>,
    pub slices: Vec<SliceExpr>,
}

unsafe impl Sync for Machine {}

unsafe impl Sync for Context {}

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
        println!("[*] Loading: {:?}...", filepath);

        let mut f = File::open(filepath).unwrap();

        f.seek(SeekFrom::End(0)).unwrap();
        let size = f.stream_position().unwrap() as usize;
        f.seek(SeekFrom::Start(0)).unwrap();

        let mut buf: Vec<u8> = Vec::with_capacity(size);
        buf.resize(size, 0);
        f.read(&mut buf).unwrap();

        let file: MachineFile = rmp_serde::from_slice(&buf).unwrap();

        let slices = file.cache.to_vec();

        println!("[*] Ready!");

        Machine {
            initial: file.initial.clone(),
            slices
        }
    }

    pub fn save(&self, filepath: &str) {
        let cache = SliceCache::from_vec(&self.slices);

        let file = MachineFile {
            initial: self.initial.clone(),
            cache
        };

        let mut f = File::create(filepath).unwrap();

        let ser = rmp_serde::to_vec(&file).unwrap();
        f.write(&ser).unwrap();
    }

    pub fn run(&self) {
        println!("[*] Running...");

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

            // let unfilled = ctx.unfilled(t);
            // if unfilled.len() > 0 {
            //     panic!();
            // }

            if cref.get_mut().shift() {
                break;
            }
            t += 1;
        }
    }

    // pub fn trace_run(&self, steps: usize) {
    //     let mut ctx = Context::new_sized(self.slices.len(), TEMPORAL);
    //     for i in 0..self.initial.len() {
    //         ctx.set(i, 0, self.initial[i])
    //     }

    //     for i in 0..steps {
            
    //         // let bar = ProgressBar::new(self.slices.len() as u64);
    //         for k in 0..self.slices.len() {
    //             if let Ok(v) = self.slices[k].eval(&mut ctx, i) {
    //                 ctx.set(k, i+1, v);
    //             } else {
    //                 panic!("Bad slice: {:?}", k);
    //             }
    //             // bar.inc(1);
    //         }
    //         // bar.finish();

    //         let pc = ctx.get_reg(0x4000, i+1).unwrap();
    //         print!("[{i}] pc={:?} ", pc);

    //         for j in 0..8 {
    //             let reg = ctx.get_reg(0x4010 + (j * 16), i+1).unwrap();
    //             print!("r{j}={:?} ", reg);
    //         }
    //         println!("");

    //         if ctx.shift() {
    //             break;
    //         }
    //     }
    // }

    pub fn shuffle(&mut self, seed: usize) {
        let size = self.initial.len();

        let mut mseed = [0u8; 32];
        for i in 0..8 {
            mseed[i] = ((seed >> (i * 8)) & 0xff) as u8;
        }

        let mut rng = StdRng::from_seed(mseed);
        
        let mut indexes: Vec<usize> = (0..size).collect();
        indexes.shuffle(&mut rng);

        let mut reverse_index: Vec<usize> = Vec::new();
        reverse_index.resize(size, 0usize);
        for i in 0..size {
            reverse_index[indexes[i]] = i;
        }

        println!("to cache...");
        let mut cache = SliceCache::from_vec(&self.slices);

        println!("shuffle...");
        for i in 0..cache.slices.len() {
            let slice = &mut cache.slices[i];

            match slice {
                SliceExpr::BitRef(Index {r}) => {
                    cache.slices[i] = SliceExpr::BitRef(Index { r: reverse_index[*r] })
                },
                _ => {}
            }
        }

        println!("from cache...");
        self.slices = cache.to_vec();

        let mut initial_s = Vec::new();
        let mut slices_s = Vec::new();

        for i in 0..self.initial.len() {
            initial_s.push(self.initial[indexes[i]]);
            slices_s.push(self.slices[indexes[i]].clone());
        }
        self.initial = initial_s;
        self.slices = slices_s;

        println!("done...");
    }

}

