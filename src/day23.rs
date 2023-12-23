use std::collections::VecDeque;

use anyhow::Result;
use pathfinding::prelude::bfs_reach;

use crate::grid::{CellP, Grid};

pub fn run(input: &str) -> Result<String> {
    let p1 = problem(input)?;
    Ok(format!("{p1}"))
}

fn problem(input: &str) -> Result<usize> {
    let grid = Grid::parse(input)?;

    const NYV: u16 = u16::MAX;
    let mut vis = Grid::new(grid.dimensions(), NYV);

    let mut fifo = VecDeque::from([((1, 0), 0)]);
    while let Some((p, n)) = fifo.pop_front() {
        *vis.get_mut(p).unwrap() = n;
        for q in next_steps(&grid, p) {
            if vis.get(q).copied().unwrap_or(0) == NYV {
                fifo.push_back((q, n + 1))
            }
        }
    }

    for (gr, vr) in std::iter::zip(grid.rows(), vis.rows()) {
        let v = std::iter::zip(gr.iter(), vr.iter())
            .map(|(&g, &v)| {
                if g == b'.' && v != NYV {
                    b'a' + (v % 26) as u8
                } else {
                    g
                }
            })
            .collect::<Vec<_>>();
        println!("{}", String::from_utf8_lossy(&v));
    }

    Ok(0)
}

fn nodes_reachable(grid: &Grid<u8>, from: CellP) -> Vec<(CellP, usize)> {
    let mut results = vec![];

    let (dx, dy) = grid.dimensions();
    let goal = (dx - 2, dy - 1);

    const NYV: u16 = u16::MAX;
    let mut vis = Grid::new(grid.dimensions(), NYV);

    let mut fifo = VecDeque::from([((1, 0), 0)]);
    while let Some((p, n)) = fifo.pop_front() {
        *vis.get_mut(p).unwrap() = n;
        for q in next_steps(&grid, p) {
            if vis.get(q).copied().unwrap_or(0) == NYV {
                let n = n + 1;
                if q == goal || is_ice(*grid.get(q).unwrap_or(&b'.')) {
                    results.push((q, n as usize));
                } else {
                    fifo.push_back((q, n));
                }
            }
        }
    }

    results
}

fn is_ice(c: u8) -> bool {
    DIRS.iter().find(|(_, dc)| dc == &c).is_some()
}

fn next_steps(grid: &Grid<u8>, p: CellP) -> impl Iterator<Item = CellP> + '_ {
    DIRS.iter()
        .copied()
        .enumerate()
        .filter_map(move |(i, (d, c))| {
            let pc = *grid.get(p)?;
            let p_ok = pc == b'.' || pc == c;

            let q = (p.0 + d.0, p.1 + d.1);
            let qc = *grid.get(p)?;
            let q_ok = qc != b'#' && qc != DIRS[opposite_dir(i)].1;

            (p_ok && q_ok).then_some(q)
        })
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct BfsNode {
    p: (i32, i32),
    skids: IceBuf,
    n: usize,
}

impl BfsNode {
    fn start(p: (i32, i32)) -> Self {
        Self {
            p,
            skids: IceBuf::new(),
            n: 0,
        }
    }

    fn step(&self, grid: &Grid<u8>, dir_idx: usize) -> Option<Self> {
        let p = self.p;
        let (d, c) = DIRS[dir_idx];

        let q = (p.0 + d.0, p.1 + d.1);

        let m0 = *grid.get(p).unwrap();
        if m0 != b'.' {
            return (m0 == c).then(|| self.step_impl(q, Some(dir_idx)));
        }

        let m = *grid.get(q)?;
        (m != b'#' && m != DIRS[opposite_dir(dir_idx)].1).then_some(self.step_impl(q, None))
    }

    fn step_impl(&self, newp: (i32, i32), add_skid: Option<usize>) -> Self {
        let mut skids = self.skids;
        if let Some(skid) = add_skid {
            skids.push(skid);
        }

        Self {
            p: newp,
            skids,
            n: self.n + 1,
        }
    }
}

const DIRS: [((i32, i32), u8); 4] = [
    ((-1, 0), b'<'),
    ((1, 0), b'>'),
    ((0, -1), b'^'),
    ((0, 1), b'v'),
];

fn opposite_dir(idx: usize) -> usize {
    idx ^ 0x1
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct IceBuf {
    ar: [u8; 40],
    l: usize,
}

impl IceBuf {
    fn new() -> Self {
        Self { ar: [0; 40], l: 0 }
    }

    fn push(&mut self, n: usize) {
        let (i, s) = (self.l / 4, self.l % 4);
        self.ar[i] &= ((n % 4) as u8) << s;
        self.l += 1;
    }
}

impl std::fmt::Display for IceBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let values = self
            .ar
            .iter()
            .flat_map(|b| (0..4).map(move |i| (b >> (i % 4)) & 3));
        for i in values.take(self.l) {
            write!(f, "{}", DIRS[i as usize].1 as char)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let sample = "\
#.#####################
#.......#########...###
#######.#########.#.###
###.....#.>.>.###.#.###
###v#####.#v#.###.#.###
###.>...#.#.#.....#...#
###v###.#.#.#########.#
###...#.#.#.......#...#
#####.#.#.#######.#.###
#.....#.#.#.......#...#
#.#####.#.#.#########v#
#.#...#...#...###...>.#
#.#.#v#######v###.###v#
#...#.>.#...>.>.#.###.#
#####v#.#.###v#.#.###.#
#.....#...#...#.#.#...#
#.#########.###.#.#.###
#...###...#...#...#.###
###.###.#.###v#####v###
#...#...#.#.>.>.#.>.###
#.###.###.#.###.#.#v###
#.....###...###...#...#
#####################.#
";
        assert_eq!(problem(sample).ok(), Some(94));
    }
}
