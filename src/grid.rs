use std::collections::HashSet;

use anyhow::{bail, Result};

// Cell x and y coordinates
pub type CellP = (i32, i32);

#[allow(unused)]
pub const STEPS: &[CellP; 4] = &[(0, -1), (0, 1), (-1, 0), (1, 0)];

#[allow(unused)]
pub const DIRS: &[Dir; 4] = &[Dir::North, Dir::South, Dir::West, Dir::East];

// Cardinal directions
#[allow(unused)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Dir {
    North,
    South,
    East,
    West,
}

#[allow(unused)]
impl Dir {
    pub fn from_xy((dx, dy): CellP) -> Option<Self> {
        use std::cmp::Ordering::*;
        if dx == 0 {
            match dy.cmp(&0) {
                Less => Some(Self::North),
                Equal => None,
                Greater => Some(Self::South),
            }
        } else if dy == 0 {
            match dx.cmp(&0) {
                Less => Some(Self::West),
                Equal => None,
                Greater => Some(Self::East),
            }
        } else {
            None
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
        }
    }

    pub fn index(self) -> u8 {
        match self {
            Self::North => 0,
            Self::South => 1,
            Self::East => 2,
            Self::West => 3,
        }
    }

    pub fn step(self, x: i32) -> CellP {
        match self {
            Self::North => (0, -x),
            Self::South => (0, x),
            Self::East => (x, 0),
            Self::West => (-x, 0),
        }
    }
}

#[derive(Clone)]
pub struct Grid<T> {
    dx: i32,
    dy: i32,
    m: Vec<T>,
}

#[allow(unused)]
impl<T> Grid<T> {
    pub fn positions(&self) -> impl Iterator<Item = CellP> + '_ {
        (0..self.dy).flat_map(|y| (0..self.dx).map(move |x| (x, y)))
    }

    pub fn iter(&self) -> impl Iterator<Item = (CellP, &T)> {
        std::iter::zip(self.positions(), self.m.iter())
    }

    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.m.iter()
    }

    pub fn rows(&self) -> impl Iterator<Item = &[T]> {
        self.m.chunks(self.dx as usize)
    }

    pub fn as_slice(&self) -> &[T] {
        &self.m
    }

    pub fn dimensions(&self) -> (i32, i32) {
        (self.dx, self.dy)
    }

    pub fn to_xy(&self, p: usize) -> Option<CellP> {
        (p < self.m.len()).then(|| {
            let i = p as i32;
            (i % self.dx, i / self.dx)
        })
    }

    pub fn to_index(&self, p: CellP) -> Option<usize> {
        self.is_inside(p).then(|| {
            let (px, py) = p;
            (px + py * self.dx) as usize
        })
    }

    pub fn is_inside(&self, p: CellP) -> bool {
        let (px, py) = p;
        px >= 0 && px < self.dx && py >= 0 && py < self.dy
    }

    pub fn get(&self, p: CellP) -> Option<&T> {
        self.to_index(p).map(|i| &self.m[i])
    }

    pub fn get_mut(&mut self, p: CellP) -> Option<&mut T> {
        self.to_index(p).map(|i| &mut self.m[i])
    }
}

#[allow(unused)]
impl<T: Clone> Grid<T> {
    pub fn new(dims: CellP, v: T) -> Self {
        let (dx, dy) = dims;
        Self {
            dx,
            dy,
            m: vec![v; (dx * dy) as usize],
        }
    }

    pub fn fill_block(&mut self, p0: CellP, p1: CellP, fillc: T) {
        let x0 = p0.0.min(p1.0).max(0);
        let x1 = p0.0.max(p1.0).min(self.dx);
        let y0 = p0.1.min(p1.1).max(0);
        let y1 = p0.1.max(p1.1).min(self.dy);

        let mut s = (x0 + y0 * self.dx) as usize;
        let w = (x1 - x0) as usize;
        for _ in y0..y1 {
            self.m[s..s + w].fill(fillc.clone());
            s += self.dx as usize;
        }
    }
}

#[allow(unused)]
impl Grid<u8> {
    pub fn parse(input: &str) -> Result<Self> {
        let (dx, m) = input
            .lines()
            .try_fold((0, Vec::new()), |(dx, mut v), line| {
                let bytes = line.as_bytes();
                if !v.is_empty() && dx != bytes.len() {
                    bail!("invalid line");
                }
                v.extend_from_slice(bytes);
                Ok((bytes.len(), v))
            })?;
        let dy = m.len() / dx;
        Ok(Self {
            dx: dx as i32,
            dy: dy as i32,
            m,
        })
    }

    pub fn show(&self) {
        for row in self.m.chunks(self.dx as usize) {
            println!("{}", String::from_utf8_lossy(row));
        }
    }
}

#[allow(unused)]
impl<T: PartialEq> Grid<T> {
    pub fn find(&self, what: &T) -> Option<CellP> {
        self.m
            .iter()
            .position(|c| c == what)
            .and_then(|p| self.to_xy(p))
    }
}

#[allow(unused)]
impl<T: PartialEq + Clone> Grid<T> {
    pub fn flood<P>(&mut self, start: CellP, value: T, mut pred: P)
    where
        P: FnMut(&T) -> bool,
    {
        if !self.is_inside(start) {
            return;
        }

        let mut stack = vec![start];
        let mut visited = HashSet::new();
        while let Some(p) = stack.pop() {
            *self.get_mut(p).unwrap() = value.clone();
            for &d in STEPS {
                let q = (p.0 + d.0, p.1 + d.1);
                if self.is_inside(q) && !visited.contains(&q) && pred(self.get(q).unwrap()) {
                    visited.insert(q);
                    stack.push(q);
                }
            }
        }
    }
}
