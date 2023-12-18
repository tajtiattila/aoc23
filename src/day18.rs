use anyhow::{anyhow, Result};

use crate::grid::{Dir, Grid};

pub fn run(input: &str) -> Result<String> {
    Ok(format!("{} {}", part1(input)?, part2(input)?))
}

fn part1(input: &str) -> Result<usize> {
    calc_sparse(input, Plan::parse)
}

fn part2(input: &str) -> Result<usize> {
    calc_sparse(input, Plan::parse_fix)
}

fn calc_sparse(input: &str, mut f: impl FnMut(&str) -> Option<Plan>) -> Result<usize> {
    let plan = input
        .lines()
        .map(|line| f(line).ok_or_else(|| anyhow!("invalid line {line}")))
        .collect::<Result<Vec<_>>>()?;

    let mut pts = vec![];
    let mut p = (0, 0);
    for cmd in &plan {
        pts.push(p);
        let d = cmd.dir.step(1);
        let d = (d.0 as i64 * cmd.len as i64, d.1 as i64 * cmd.len as i64);
        p.0 += d.0;
        p.1 += d.1;
    }

    let mut sparse = SparseGrid::from_control_points(b'.', pts.iter().copied());

    let mut p = (0, 0);
    for cmd in &plan {
        let d = cmd.dir.step(1);
        let d = (d.0 as i64 * cmd.len as i64, d.1 as i64 * cmd.len as i64);

        let q00 = p;
        let q01 = (p.0 + 1, p.1 + 1);

        p.0 += d.0;
        p.1 += d.1;

        let q10 = p;
        let q11 = (p.0 + 1, p.1 + 1);

        let qts = [q00, q01, q10, q11];
        let x0 = qts.iter().map(|q| q.0).min().unwrap();
        let x1 = qts.iter().map(|q| q.0).max().unwrap();
        let y0 = qts.iter().map(|q| q.1).min().unwrap();
        let y1 = qts.iter().map(|q| q.1).max().unwrap();
        sparse.fill_block((x0, y0), (x1, y1), b'#');
    }

    sparse.flood_from_outside(b'+', |&b| b != b'#');

    let dbg = cfg!(test) || crate::Cli::global().verbose;
    if dbg {
        sparse.grid.show();
        println!();
    }

    Ok(sparse.count_areas(|&b| b != b'+'))
}

struct SparseGrid {
    vx: Vec<i64>,
    vy: Vec<i64>,
    grid: Grid<u8>,
}

impl SparseGrid {
    fn from_control_points(fillc: u8, it: impl Iterator<Item = (i64, i64)>) -> Self {
        let mut vx = vec![];
        let mut vy = vec![];
        for p in it {
            vx.push(p.0);
            vx.push(p.0 + 1);

            vy.push(p.1);
            vy.push(p.1 + 1);
        }

        vx.sort();
        vx.dedup();

        vy.sort();
        vy.dedup();

        let extend = |v: &mut Vec<_>| {
            let z0 = v.first().unwrap() - 1;
            let z1 = v.last().unwrap() + 1;
            v.insert(0, z0);
            v.push(z1);
        };

        extend(&mut vx);
        extend(&mut vy);

        let dx = (vx.len() - 1) as i32;
        let dy = (vy.len() - 1) as i32;
        let grid = Grid::new((dx, dy), fillc);

        Self { vx, vy, grid }
    }

    fn fill_block(&mut self, p0: (i64, i64), p1: (i64, i64), fillc: u8) {
        let x0 = p0.0.min(p1.0);
        let x1 = p0.0.max(p1.0);
        let xr = Self::map_range(&self.vx, x0, x1);

        let y0 = p0.1.min(p1.1);
        let y1 = p0.1.max(p1.1);
        let yr = Self::map_range(&self.vy, y0, y1);

        self.grid
            .fill_block((xr.start, yr.start), (xr.end, yr.end), fillc);
    }

    fn flood_from_outside(&mut self, fillc: u8, pred: impl FnMut(&u8) -> bool) {
        self.grid.flood((0, 0), fillc, pred);
    }

    fn count_areas(&self, mut pred: impl FnMut(&u8) -> bool) -> usize {
        let mx = self.vx.len() - 1;
        let my = self.vy.len() - 1;
        let mut total = 0;
        for x in 0..mx {
            let bx = (self.vx[x + 1] - self.vx[x]) as usize;
            for y in 0..my {
                let by = (self.vy[y + 1] - self.vy[y]) as usize;
                if pred(self.grid.get((x as i32, y as i32)).unwrap()) {
                    total += bx * by;
                }
            }
        }
        total
    }

    fn map_range(v: &[i64], x0: i64, x1: i64) -> std::ops::Range<i32> {
        let i0 = v.partition_point(|&x| x < x0) as i32;
        let i1 = v.partition_point(|&x| x < x1) as i32;
        i0..i1
    }
}

struct Plan {
    dir: Dir,
    len: usize,
    rgb: u32,
}

impl Plan {
    fn parse(s: &str) -> Option<Plan> {
        let mut it = s.split_whitespace();
        let dir = match it.next()? {
            "L" => Dir::West,
            "R" => Dir::East,
            "U" => Dir::North,
            "D" => Dir::South,
            _ => return None,
        };

        let len = it.next()?.parse().ok()?;

        let c = it.next()?.strip_prefix("(#")?.strip_suffix(')')?;
        let rgb = u32::from_str_radix(c, 16).ok()?;

        it.next().is_none().then_some(Plan { dir, len, rgb })
    }

    fn parse_fix(s: &str) -> Option<Plan> {
        let Plan { rgb, .. } = Self::parse(s)?;
        let dir = match rgb & 0xF {
            0 => Dir::East,
            1 => Dir::South,
            2 => Dir::West,
            3 => Dir::North,
            _ => return None,
        };
        let len = ((rgb & 0xFFFFF0) >> 4) as usize;
        Some(Self { dir, len, rgb })
    }
}
