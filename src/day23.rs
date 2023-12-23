use anyhow::Result;
use pathfinding::prelude::bfs_reach;

use crate::grid::Grid;

pub fn run(input: &str) -> Result<String> {
    let p1 = problem(input)?;
    Ok(format!("{p1}"))
}

fn problem(input: &str) -> Result<usize> {
    let grid = Grid::parse(input)?;

    let succ = |node: &BfsNode| {
        (0..DIRS.len())
            .filter_map(|i| node.step(&grid, i))
            .collect::<Vec<_>>()
    };

    for r in bfs_reach(BfsNode::start((1, 0)), succ) {
        println!("{r:?}");
    }

    Ok(0)
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
