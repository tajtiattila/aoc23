use std::collections::{HashMap, VecDeque};

use anyhow::{anyhow, Result};

use crate::grid::{CellP, Grid};

pub fn run(input: &str) -> Result<String> {
    let (p1, p2) = problem_j(input)?;
    Ok(format!("{p1} {p2}"))
}

fn problem(input: &str) -> Result<(usize, usize)> {
    let grid = Grid::parse(input)?;
    Ok((longest_path(&grid, true)?, longest_path(&grid, false)?))
}

fn problem_j(input: &str) -> Result<(usize, usize)> {
    let grid = Grid::parse(input)?;
    Ok((
        max_using_junctions(&grid, true)?,
        max_using_junctions(&grid, false)?,
    ))
}

fn max_using_junctions(grid: &Grid<u8>, slippery: bool) -> Result<usize> {
    let (dx, dy) = grid.dimensions();

    let start = (1, 0);
    let goal = (dx - 2, dy - 1);

    let junctions = || {
        (1..(dx - 1))
            .flat_map(|x| (1..(dy - 1)).map(move |y| (x, y)))
            .filter(|&p| is_junction(grid, p))
    };

    let pt_indices = junctions()
        .enumerate()
        .map(|(i, p)| (p, i))
        .collect::<HashMap<_, _>>();
    let pt_mask = |p| (1 as u64) << pt_indices.get(&p).unwrap();

    let graph = std::iter::once(start)
        .chain(junctions())
        .map(|p| (p, longest_paths_to_junctions(&grid, p, slippery)))
        .collect::<HashMap<_, _>>();

    let mut stack = vec![(start, 0, 0)];
    let mut result = None;

    while let Some((pt, vis_mask, p_len)) = stack.pop() {
        for &(next, n_len) in graph.get(&pt).unwrap() {
            let len = p_len + n_len;
            if next == goal {
                if len > result.unwrap_or(0) {
                    result = Some(len);
                }
            } else {
                let next_mask = pt_mask(next);
                if vis_mask & next_mask == 0 {
                    stack.push((next, vis_mask | next_mask, len));
                }
            }
        }
    }

    result.ok_or_else(|| anyhow!("path not found"))
}

fn longest_paths_to_junctions(grid: &Grid<u8>, from: CellP, slippery: bool) -> Vec<(CellP, usize)> {
    let goal = grid_goal(&grid);

    let is_junc_goal = |p| p == goal || is_junction(grid, p);

    let mut paths = next_steps(grid, from, slippery)
        .map(|p| vec![p])
        .collect::<Vec<_>>();

    let mut results = Vec::<(CellP, usize)>::new();

    while let Some(mut path) = paths.pop() {
        let (last, cur) = match path.len() {
            0 => unreachable!(),
            1 => (from, path[0]),
            n => (path[n - 2], path[n - 1]),
        };

        let mut it = next_steps(grid, cur, slippery).filter(|&p| p != last);
        if let Some(next) = it.next() {
            path.push(next);
            if is_junc_goal(next) {
                if let Some((_, l)) = results.iter_mut().find(|(p, _)| *p == next) {
                    *l = path.len().max(*l);
                } else {
                    results.push((next, path.len()));
                }
            } else {
                paths.push(path);
            }
        }

        assert!(it.next().is_none());
    }

    results
}

fn is_junction(grid: &Grid<u8>, p: CellP) -> bool {
    grid.get(p).unwrap_or(&b'#') != &b'#' && {
        let c = DIRS
            .iter()
            .filter(|(d, _)| {
                let p = (p.0 + d.0, p.1 + d.1);
                grid.get(p).unwrap_or(&b'#') != &b'#'
            })
            .count();
        c > 2
    }
}

fn longest_path(grid: &Grid<u8>, slippery: bool) -> Result<usize> {
    let goal = grid_goal(&grid);
    let mut result = Err(anyhow!("path not found"));

    let mut fifo = VecDeque::from([PathEntry::from(grid.dimensions(), (1, 0))]);
    while let Some(e) = fifo.pop_front() {
        for p in next_steps(grid, e.p, slippery) {
            if e.vis.get(p) == Some(false) {
                if p == goal {
                    let n = e.n + 1;
                    if &n > result.as_ref().unwrap_or(&0) {
                        result = Ok(n);
                        println!("{n}");
                    }
                } else {
                    fifo.push_back(e.next(p));
                }
            }
        }
    }

    result
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct PathEntry {
    p: CellP,
    n: usize,
    vis: VisGrid,
}

impl PathEntry {
    fn from(dim: CellP, p: CellP) -> Self {
        let mut vis = VisGrid::new(dim);
        vis.set(p, true);
        Self { p, vis, n: 0 }
    }

    fn next(&self, p: CellP) -> Self {
        let mut vis = self.vis.clone();
        vis.set(p, true);
        Self {
            p,
            vis,
            n: self.n + 1,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct VisGrid {
    dim: CellP,
    v: Vec<u8>,
}

impl VisGrid {
    fn new(dim: CellP) -> Self {
        Self {
            dim,
            v: vec![0; (dim.0 * dim.1) as usize],
        }
    }

    fn get(&self, p: CellP) -> Option<bool> {
        self.to_idx_mask(p).map(|(i, m)| self.v[i] & m != 0)
    }

    fn set(&mut self, p: CellP, v: bool) {
        if let Some((i, m)) = self.to_idx_mask(p) {
            let p = &mut self.v[i];
            if v {
                *p |= m;
            } else {
                *p &= !m;
            }
        }
    }

    fn to_idx_mask(&self, p: CellP) -> Option<(usize, u8)> {
        ((0..self.dim.0).contains(&p.0) && (0..self.dim.1).contains(&p.1)).then(|| {
            let i = (p.0 + p.1 * self.dim.0) as usize;
            (i / 8, 1 << (i % 8))
        })
    }
}

fn grid_goal(grid: &Grid<u8>) -> CellP {
    let (dx, dy) = grid.dimensions();
    (dx - 2, dy - 1)
}

fn next_steps(grid: &Grid<u8>, p: CellP, slippery: bool) -> impl Iterator<Item = CellP> + '_ {
    DIRS.iter()
        .copied()
        .enumerate()
        .filter_map(move |(i, (d, c))| {
            let q = (p.0 + d.0, p.1 + d.1);
            let qc = *grid.get(q)?;
            if slippery {
                let pc = *grid.get(p)?;
                let p_ok = pc == b'.' || pc == c;

                let q_ok = qc != b'#' && qc != DIRS[opposite_dir(i)].1;

                (p_ok && q_ok).then_some(q)
            } else {
                (qc != b'#').then_some(q)
            }
        })
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
        assert_eq!(problem(sample).ok(), Some((94, 154)));
        println!("ok");

        assert_eq!(problem_j(sample).ok(), Some((94, 154)));
    }
}
