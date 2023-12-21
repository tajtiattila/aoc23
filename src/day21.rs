use anyhow::{anyhow, Result};
use std::collections::VecDeque;

use crate::grid::{Grid, STEPS};

pub fn run(input: &str) -> Result<String> {
    Ok(format!("{}", part1(input, 64)?))
}

fn part1(input: &str, nsteps: usize) -> Result<usize> {
    let grid = Grid::parse(input)?;

    const NYV: usize = usize::MAX; // not yet visited
    let mut vis = Grid::new(grid.dimensions(), NYV);

    let start = grid
        .find(&b'S')
        .ok_or_else(|| anyhow!("no starting position"))?;
    *vis.get_mut(start).unwrap() = 0;

    let mut fifo = VecDeque::from([(0, start)]);
    while let Some((n, p)) = fifo.pop_front() {
        if n < nsteps {
            for &d in STEPS {
                let q = (p.0 + d.0, p.1 + d.1);
                if *grid.get(q).unwrap_or(&b'#') != b'#' && *vis.get(q).unwrap() == NYV {
                    *vis.get_mut(q).unwrap() = n + 1;
                    fifo.push_back((n + 1, q));
                }
            }
        }
    }

    Ok(vis.values().filter(|&&n| n != NYV && (n % 2 == 0)).count())
}
