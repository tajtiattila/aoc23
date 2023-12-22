use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Result};

use crate::quadmap;

pub fn run(input: &str) -> Result<String> {
    let p1 = part1(input)?;
    let p2 = part2(input)?;
    Ok(format!("{p1} {p2}"))
}

fn part1(input: &str) -> Result<usize> {
    let mut bricks = input
        .lines()
        .map(Brick::parse)
        .collect::<Result<Vec<_>>>()?;

    let FallResult {
        supports,
        supported_by,
        n_fallen: _,
    } = drop_bricks(&mut bricks);

    let can_remove_single = (0..bricks.len())
        .filter(|i| {
            if let Some(bricks) = supports.get(&i) {
                bricks
                    .iter()
                    .all(|i| supported_by.get(i).unwrap().len() > 1)
            } else {
                true
            }
        })
        .count();

    Ok(can_remove_single)
}

fn part2(input: &str) -> Result<usize> {
    let mut original_bricks = input
        .lines()
        .map(Brick::parse)
        .collect::<Result<Vec<_>>>()?;

    drop_bricks(&mut original_bricks);

    Ok((0..original_bricks.len())
        .map(|i| {
            let mut bricks = original_bricks.clone();
            bricks.remove(i);
            drop_bricks(&mut bricks).n_fallen
        })
        .sum())
}

fn drop_bricks(bricks: &mut [Brick]) -> FallResult {
    bricks.sort_unstable_by_key(Brick::bottom);

    let mut highest = quadmap::Map::new((0, None));

    let mut supports = HashMap::<usize, HashSet<usize>>::new();
    let mut supported_by = HashMap::<usize, HashSet<usize>>::new();

    let mut n_fallen = 0;

    for (i, brk) in bricks.iter_mut().enumerate() {
        let Vec3(x0, y0, _) = brk.l;
        let Vec3(x1, y1, _) = brk.r;
        let brk_flat = || (x0..=x1).flat_map(|x| (y0..=y1).map(move |y| (x, y)));
        let new_bottom = brk_flat().map(|p| highest.at(p).0).max().unwrap() + 1;

        let drop = brk.bottom() - new_bottom;
        brk.l.2 = new_bottom;
        brk.r.2 -= drop;

        if drop > 0 {
            n_fallen += 1;
        }

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

    FallResult {
        supports,
        supported_by,
        n_fallen,
    }
}

struct FallResult {
    supports: HashMap<usize, HashSet<usize>>,
    supported_by: HashMap<usize, HashSet<usize>>,
    n_fallen: usize,
}

#[derive(Clone)]
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

#[derive(Clone)]
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
        assert_eq!(part2(sample).ok(), Some(7));
    }
}
