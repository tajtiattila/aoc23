use std::cmp::{max, min};
use std::collections::HashMap;

const SIZE: i32 = 16; // x/y size of one block

#[allow(unused)]
#[derive(Debug)]
pub struct Map<T> {
    zero: T,
    m: HashMap<(i32, i32), [T; (SIZE * SIZE) as usize]>,
    bounds: Bounds,
}

#[allow(unused)]
impl<T: Copy> Map<T> {
    pub fn new(zero: T) -> Map<T> {
        Map {
            zero,
            m: HashMap::new(),
            bounds: Bounds::new(),
        }
    }

    pub fn at(&self, p: (i32, i32)) -> &T {
        let (k, o) = self.mpos(p);
        if let Some(v) = self.m.get(&k) {
            &v[o]
        } else {
            &self.zero
        }
    }

    pub fn at_mut(&mut self, p: (i32, i32)) -> &mut T {
        self.bounds.extend_one(p);
        let (k, o) = self.mpos(p);
        let v = self
            .m
            .entry(k)
            .or_insert([self.zero; (SIZE * SIZE) as usize]);
        &mut v[o]
    }

    pub fn bounds(&self) -> &Bounds {
        &self.bounds
    }

    pub fn hline(&mut self, x0: i32, x1: i32, y: i32, item: &T) {
        let (x0, x1) = lohi(x0, x1);
        for x in x0..=x1 {
            *self.at_mut((x, y)) = *item;
        }
    }

    pub fn vline(&mut self, x: i32, y0: i32, y1: i32, item: &T) {
        let (y0, y1) = lohi(y0, y1);
        for y in y0..=y1 {
            *self.at_mut((x, y)) = *item;
        }
    }

    fn mpos(&self, p: (i32, i32)) -> ((i32, i32), usize) {
        (
            (p.0.div_euclid(SIZE), p.1.div_euclid(SIZE)),
            (p.0.rem_euclid(SIZE) + SIZE * p.1.rem_euclid(SIZE)) as usize,
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Bounds {
    pub min: (i32, i32),
    pub max: (i32, i32),
}

impl Bounds {
    pub fn new() -> Bounds {
        Self {
            min: (0, 0),
            max: (0, 0),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.min.0 == self.max.0 || self.min.1 == self.max.1
    }

    pub fn extend_one(&mut self, p: (i32, i32)) {
        if self.is_empty() {
            self.min = p;
            self.max = (p.0 + 1, p.1 + 1);
        } else {
            self.min.0 = min(self.min.0, p.0);
            self.min.1 = min(self.min.1, p.1);
            self.max.0 = max(self.max.0, p.0 + 1);
            self.max.1 = max(self.max.1, p.1 + 1);
        }
    }
}

fn lohi<T: std::cmp::Ord>(a: T, b: T) -> (T, T) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}
