use std::{io::{Read, Write}, sync::Mutex};

pub struct CBit {
    pub is_set: bool,
    pub val: bool
}

pub struct CChar {
    pub is_set: bool,
    pub val: u8
}

pub struct Context {
    pub memory: Vec<Vec<CBit>>,
    pub input: Mutex<Vec<CChar>>,
    pub output: Vec<Vec<CBit>>,

    pub outer: usize,
    pub inner: usize,

    pub m_size: usize,
    pub t_size: usize,
}

impl Context {
    pub fn new_sized(m_size: usize, t_size: usize) -> Self {
        let mut memory = Vec::with_capacity(t_size);
        let mut input = Vec::with_capacity(t_size);
        let mut output = Vec::with_capacity(t_size);
        for _ in 0..t_size {
            let mut v = Vec::with_capacity(m_size);
            for _ in 0..m_size {
                v.push(CBit { is_set: false, val: false });
            }
            memory.push(v);
            input.push(CChar { is_set: false, val: 0 });
            let mut o = Vec::with_capacity(8);
            for _ in 0..8 {
                o.push(CBit { is_set: false, val: false });
            }
            output.push(o);
        }
        Context {
            memory,
            input: Mutex::new(input),
            output,
            outer: 0,
            inner: 0,
            t_size,
            m_size
        }
    }

    pub fn unfilled(&self, k: usize) -> Vec<usize> {
        if k - self.outer >= self.t_size {
            return vec![];
        }

        let m = (self.inner + (k - self.outer)) % self.t_size;

        self.memory[m].iter().enumerate()
            .filter(|(_,b)| !b.is_set)
            .map(|(a,_)| a)
            .collect()
    }

    pub fn shift(&mut self) -> bool {
        let s = self.memory[self.inner].iter().all(|c| c.is_set);
        if s {
            for i in 0..self.m_size {
                self.memory[self.inner][i].is_set = false;
            }
            self.input.lock().unwrap()[self.inner].is_set = false;

            if self.output[self.inner].iter().all(|x| x.is_set) {
                let c = self.output[self.inner].iter()
                    .rfold(0, |acc,x| (acc << 1) | (x.val as u8));

                if c == 0xff {
                    return true;
                }
                std::io::stdout().write(&[c]).unwrap();
                std::io::stdout().flush().unwrap();
            }

            for i in 0..8 {
                self.output[self.inner][i].is_set = false;
            }
            self.outer += 1;
            self.inner = (self.inner + 1) % self.t_size;
        }
        false
    }

    pub fn get(&self, idx: usize, k: usize) -> Option<bool> {
        if k - self.outer >= self.t_size {
            return None;
        }

        let m = (self.inner + (k - self.outer)) % self.t_size;

        let c = &self.memory[m][idx];
        c.is_set.then(|| c.val)
    }

    pub fn set(&mut self, idx: usize, k: usize, v: bool) {
        if k - self.outer >= self.t_size {
            return;
        }

        let m = (self.inner + (k - self.outer)) % self.t_size;
        self.memory[m][idx] = CBit { is_set: true, val: v };
    }

    pub fn input(&mut self, idx: usize, k: usize) -> Option<bool> {
        if k - self.outer >= self.t_size {
            return None;
        }

        let m = (self.inner + (k - self.outer)) % self.t_size;

        let c = &mut self.input.lock().unwrap()[m];
        if !c.is_set {
            let mut buf = [0u8; 1];
            let n = std::io::stdin().read(&mut buf);

            if let Ok(1usize) = n {
                c.is_set = true;
                c.val = buf[0];
            }
        }

        c.is_set.then(|| (c.val >> idx) & 1 == 1)
    }

    pub fn output(&mut self, idx: usize, val: bool, k: usize) {
        if k - self.outer >= self.t_size {
            return;
        }

        let m = (self.inner + (k - self.outer)) % self.t_size;

        self.output[m][idx].is_set = true;
        self.output[m][idx].val = val;
    }
}
