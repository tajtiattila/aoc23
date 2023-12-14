use std::collections::HashSet;

use anyhow::{anyhow, bail, Result};

pub fn run(input: &str) -> Result<String> {
    Ok(format!("{} {}", p1(input)?, p2(input)?))
}

fn p1(input: &str) -> Result<usize> {
    let mut p = Platform::parse(input)?;
    p.roll(Dir::North);
    Ok(p.load())
}

fn p2(input: &str) -> Result<usize> {
    const STEPS: usize = 1_000_000_000;
    load_after(input, STEPS)
}

fn load_after(input: &str, i: usize) -> Result<usize> {
    let mut p = Platform::parse(input)?;

    let i = i.max(1);

    let mut seen = HashSet::new();

    let mut v = vec![];
    let mut last_new = 0;
    loop {
        use Dir::*;
        let l = p.load();

        v.push(l);
        if v.len() == i {
            return Ok(*v.last().unwrap());
        }

        if seen.contains(&l) {
            if v.len() > last_new + 100 {
                break;
            }
        } else {
            seen.insert(l);
            last_new = v.len();
        }

        for dir in [North, West, South, East].iter() {
            p.roll(*dir);
        }
    }

    let nrep = rfind_repeat(&v, 2).ok_or_else(|| anyhow!("can't find repeat"))?;
    let n0 = v.len() - nrep;
    let rpt = &v[n0..];

    let dbg = cfg!(test) || crate::Cli::global().verbose;
    if dbg {
        println!("{n0},{nrep} {:?}", rpt);
    }

    let j = (i - n0) % nrep;

    Ok(rpt[j])
}

fn rfind_repeat(v: &[usize], min_rpt: usize) -> Option<usize> {
    let n = v.len();
    for l in min_rpt..(n / 2) {
        let s1 = &v[(n - l)..];
        let s2 = &v[(n - 2 * l)..(n - l)];
        if s1 == s2 {
            return Some(l);
        }
    }

    None
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct Platform {
    dx: i32,
    dy: i32,
    m: Vec<u8>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Dir {
    North,
    West,
    South,
    East,
}

impl Platform {
    fn parse(input: &str) -> Result<Self> {
        let (dx, v) = input.lines().try_fold((0, vec![]), |(dx, mut v), line| {
            let b = line.as_bytes();
            if v.is_empty() {
                Ok((b.len(), b.to_vec()))
            } else {
                if b.len() != dx {
                    bail!("invalid line {line}");
                }
                v.extend_from_slice(b);
                Ok((dx, v))
            }
        })?;

        Ok(Self {
            dx: dx as i32,
            dy: (v.len() / dx) as i32,
            m: v,
        })
    }

    #[allow(unused)]
    fn show(&self) {
        self.m
            .chunks(self.dx as usize)
            .for_each(|row| println!("{}", String::from_utf8_lossy(row)));
    }

    fn roll(&mut self, dir: Dir) {
        match dir {
            Dir::North => self.roll_impl((0, 0), (0, 1), self.dy, (1, 0), self.dx),
            Dir::West => self.roll_impl((0, 0), (1, 0), self.dx, (0, 1), self.dy),
            Dir::South => self.roll_impl((0, self.dy - 1), (0, -1), self.dy, (1, 0), self.dx),
            Dir::East => self.roll_impl((self.dx - 1, 0), (-1, 0), self.dx, (0, 1), self.dy),
        }
    }

    // return load on the north
    fn load(&self) -> usize {
        self.m
            .chunks(self.dx as usize)
            .enumerate()
            .map(|(i, row)| ((self.dy as usize) - i) * row.iter().filter(|&&x| x == b'O').count())
            .sum()
    }

    fn roll_impl(
        &mut self,
        origin: (i32, i32),
        dstep: (i32, i32),
        nstep: i32,
        dslice: (i32, i32),
        nslice: i32,
    ) {
        let mut slc = origin;
        for _ in 0..nslice {
            self.roll_slice(slc, dstep, nstep);
            slc.0 += dslice.0;
            slc.1 += dslice.1;
        }
    }

    fn roll_slice(&mut self, origin: (i32, i32), dstep: (i32, i32), nstep: i32) {
        let mut p = origin.0 + origin.1 * self.dx;
        let dstep = dstep.0 + dstep.1 * self.dx;
        let mut free = p;
        for _ in 0..nstep {
            match self.m[p as usize] {
                b'#' => free = p + dstep,
                b'O' => {
                    if p != free {
                        self.m[free as usize] = b'O';
                        self.m[p as usize] = b'.';
                    }
                    free += dstep;
                }
                _ => {}
            }

            p += dstep;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let sample_src = "\
O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#....
";
        let sample_rolld_src = "\
OOOO.#.O..
OO..#....#
OO..O##..O
O..#.OO...
........#.
..#....#.#
..O..#.O.O
..O.......
#....###..
#....#....
";

        let mut sample = Platform::parse(sample_src).unwrap();
        let sample_rolld = Platform::parse(sample_rolld_src).unwrap();

        sample.roll(Dir::North);
        sample.show();
        assert_eq!(sample, sample_rolld);

        assert_eq!(p2(sample_src).ok(), Some(64));
    }
}
