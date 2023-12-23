use std::collections::VecDeque;

use anyhow::{anyhow, Result};

use crate::grid::{CellP, Grid};

pub fn run(input: &str) -> Result<String> {
    let p1 = problem(input)?;
    Ok(format!("{p1}"))
}

fn problem(input: &str) -> Result<usize> {
    let grid = Grid::parse(input)?;

    let goal = grid_goal(&grid);
    let mut result = vec![];

    struct Entry {
        p: CellP,
        vis: VisGrid,
        n: usize,
    }
    let mut fifo = VecDeque::from([Entry {
        p: (1, 0),
        vis: VisGrid::new(grid.dimensions()),
        n: 0,
    }]);
    while let Some(e) = fifo.pop_front() {
        let n = e.n + 1;
        for p in next_steps(&grid, e.p) {
            if e.vis.get(p) == Some(false) {
                if p == goal {
                    result.push(n);
                } else {
                    let mut vis = e.vis.clone();
                    vis.set(p, true);
                    fifo.push_back(Entry { p, vis, n });
                }
            }
        }
    }

    result
        .iter()
        .max()
        .copied()
        .ok_or_else(|| anyhow!("path not found"))
}

#[derive(Debug, Clone)]
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
        assert_eq!(problem(sample).ok(), Some(94));
    }
}
