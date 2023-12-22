use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Result};

use crate::quadmap;

pub fn run(input: &str) -> Result<String> {
    println!("{}", input.lines().count());
    let p1 = part1(input)?;
    Ok(format!("{p1}"))
}

fn part1(input: &str) -> Result<usize> {
    // 1174 too high
    // 1195 too high
    let mut v = input
        .lines()
        .map(Brick::parse)
        .collect::<Result<Vec<_>>>()?;

    v.sort_unstable_by_key(Brick::bottom);

    let mut highest = quadmap::Map::new((0, None));

    let mut supports = HashMap::<usize, HashSet<usize>>::new();
    let mut supported_by = HashMap::<usize, HashSet<usize>>::new();

    for (i, brk) in v.iter_mut().enumerate() {
        let Vec3(x0, y0, _) = brk.l;
        let Vec3(x1, y1, _) = brk.r;
        let brk_flat = || (x0..=x1).flat_map(|x| (y0..=y1).map(move |y| (x, y)));
        let new_bottom = brk_flat().map(|p| highest.at(p).0).max().unwrap() + 1;

        let drop = brk.bottom() - new_bottom;
        brk.l.2 = new_bottom;
        brk.r.2 -= drop;

        for p in brk_flat() {
            let m = highest.at_mut(p);
            if m.0 + 1 == brk.bottom() {
                if let Some(brk_idx) = m.1 {
                    supports.entry(brk_idx).or_default().insert(i);
                    supported_by.entry(i).or_default().insert(brk_idx);
                }
            }
            *m = (brk.top(), Some(i));
        }
    }

    let dbg = cfg!(test) || crate::Cli::global().verbose;
    if dbg {
        println!("{supports:?}");
        println!("{supported_by:?}");
    }

    let can_remove_single = (0..v.len())
        .filter(|i| {
            if let Some(v) = supports.get(&i) {
                v.iter().all(|i| supported_by.get(i).unwrap().len() > 1)
            } else {
                true
            }
        })
        .count();

    Ok(can_remove_single)
}

struct Vec3(i32, i32, i32);

impl Vec3 {
    fn parse(xyz: &str) -> Result<Self> {
        Self::parse_impl(xyz).ok_or_else(|| anyhow!("invalid Vec3: {xyz}"))
    }

    fn parse_impl(xyz: &str) -> Option<Self> {
        let mut it = xyz.split(',');

        let mut coord = || it.next().and_then(|x| x.parse().ok());
        let v = Self(coord()?, coord()?, coord()?);

        coord().is_none().then_some(v)
    }
}

struct Brick {
    l: Vec3,
    r: Vec3,
}

impl Brick {
    fn parse(brk: &str) -> Result<Self> {
        let (l, r) = brk
            .split_once('~')
            .ok_or_else(|| anyhow!("missing `~` separator in {brk}"))?;
        Ok(Self {
            l: Vec3::parse(l)?,
            r: Vec3::parse(r)?,
        })
    }

    fn top(&self) -> i32 {
        self.r.2
    }

    fn bottom(&self) -> i32 {
        self.l.2
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let sample = "\
1,0,1~1,2,1
0,0,2~2,0,2
0,2,3~2,2,3
0,0,4~0,2,4
2,0,5~2,2,5
0,1,6~2,1,6
1,1,8~1,1,9
";
        assert_eq!(part1(sample).ok(), Some(5));
    }
}
