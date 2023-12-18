use anyhow::{anyhow, Result};

use crate::grid::{Dir, Grid};

pub fn run(input: &str) -> Result<String> {
    let plan = load(input)?;
    Ok(format!("{}", part1(&plan)))
}

fn part1(plan: &[Plan]) -> usize {
    let mut v = vec![];
    let mut p = (0, 0);
    for cmd in plan {
        let d = cmd.dir.step(1);
        for _ in 0..cmd.len {
            v.push(p);
            p.0 += d.0;
            p.1 += d.1;
        }
    }

    let calc_ext = |f: fn(i32, i32) -> i32| {
        v.iter()
            .copied()
            .reduce(|a, b| (f(a.0, b.0), f(a.1, b.1)))
            .unwrap()
    };

    let z0 = calc_ext(|x, y| x.min(y));
    let z1 = calc_ext(|x, y| x.max(y));
    let dx = (z1.0 - z0.0) + 3;
    let dy = (z1.1 - z0.1) + 3;

    let mut grid = Grid::new((dx, dy), b'.');
    for p in v {
        *grid.get_mut((p.0 - z0.0 + 1, p.1 - z0.1 + 1)).unwrap() = b'#';
    }

    grid.flood((0, 0), &b'+', |&b| b != b'#');

    let dbg = cfg!(test) || crate::Cli::global().verbose;
    if dbg {
        grid.show();
    }

    grid.values().filter(|&&b| b != b'+').count()
}

fn load(input: &str) -> Result<Vec<Plan>> {
    input
        .lines()
        .map(|line| Plan::parse(line).ok_or_else(|| anyhow!("invalid line {line}")))
        .collect()
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
}
