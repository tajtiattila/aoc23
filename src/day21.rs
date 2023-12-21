use anyhow::{anyhow, bail, Result};
use std::collections::VecDeque;

use crate::grid::{CellP, Grid, STEPS};

pub fn run(input: &str) -> Result<String> {
    Ok(format!("{} {}", part1(input, 64)?, part2(input)?))
}

fn part1(input: &str, nsteps: usize) -> Result<usize> {
    let grid = Grid::parse(input)?;

    let start = grid
        .find(&b'S')
        .ok_or_else(|| anyhow!("no starting position"))?;

    let even_odd = fill(&grid, start, nsteps);
    Ok(even_odd[nsteps % 2])
}

fn part2(input: &str) -> Result<usize> {
    calc_smart(input, 26501365)
}

fn calc_smart(input: &str, nsteps: usize) -> Result<usize> {
    let grid = Grid::parse(input)?;

    let start = verify_problem(&grid, nsteps)?;

    /*

    blocks kinds:

                       Sx
        ┌─────┬─────┬─────┬─────┬─────┐
        │     │     │  #  │     │     │
        │     │     │ ### │     │     │
        │     │     │#####│     │     │
        │     │    #│#####│#    │     │
        │     │   ##│#####│##   │     │
        ├─────┼─────┼─────┼─────┼─────┤
        │     │  ###│#####│###  │     │
        │     │ ####│#####│#### │     │
        │     │#####│#####│#####│     │
        │    #│#####│#####│#####│#    │
        │   ##│#####│#####│#####│##   │
        ├─────┼─────┼─────┼─────┼─────┤
        │  ###│#####│#####│#####│###  │
        │ ####│#####│#####│#####│#### │
        │#####│#####│#####│#####│#####│ Sy
        │ ####│#####│#####│#####│#### │
        │  ###│#####│#####│#####│###  │
        ├─────┼─────┼─────┼─────┼─────┤
        │   ##│#####│#####│#####│##   │
        │    #│#####│#####│#####│#    │
        │     │#####│#####│#####│     │
        │     │ ####│#####│#### │     │
        │     │  ###│#####│###  │     │
        ├─────┼─────┼─────┼─────┼─────┤
        │     │   ##│#####│##   │     │
        │     │    #│#####│#    │     │
        │     │     │#####│     │     │
        │     │     │ ### │     │     │
        │     │     │  #  │     │     │
        └─────┴─────┴─────┴─────┴─────┘

    Layout:

        ┌─────┬─────┬─────┬─────┬─────┐
        │empty│  nw │  n  │  ne │empty│
        ├─────┼─────┼─────┼─────┼─────┤
        │  nw │  NW │ odd │  NE │  ne │
        ├─────┼─────┼─────┼─────┼─────┤
        │  w  │ odd │ even│ odd │  e  │
        ├─────┼─────┼─────┼─────┼─────┤
        │  sw │  SW │ odd │  SE │  se │
        ├─────┼─────┼─────┼─────┼─────┤
        │empty│  sw │  s  │  se │empty│
        └─────┴─────┴─────┴─────┴─────┘

    Let w = total no. of horizontal/vertical blocks (must be odd);
        h = floor(w/2).

    Even/odd shapes are based on w:

      w mod 4 = 1:                       w mod 4 = 3:
        n_odd  =  h²                       n_even =  h²
        n_even = (h-1)²                    n_odd  = (h-1)²

            · ▲ ·
          · · o · ·                          · ▲ ·
        · · o e o · ·                      · · e · ·
      · · o e o e o · ·                  · · e o e · ·
      ◀ o e o e o e o ▶                  ◀ e o e o e ▶
      · · o e o e o · ·                  · · e o e · ·
        · · o e o · ·                      · · e · ·
          · · o · ·                          · ▼ ·
            · ▼ ·

    ("Edges" marked with triangles and · above also must take w mod 4 into account).

    Let w = total no. of horizontal/vertical blocks, must be odd

    n_edge = n + s + w + e + h*(nw+ne+sw+se) + (h-1)*(NW+NW+SW+SE)

    n_total = n_odd + n_even + n_edge

         */

    let grid_dim = grid.dimensions().0;
    let half_dim = grid_dim / 2;

    let w = (2 * nsteps + 1) / grid_dim as usize;
    let h = w / 2;

    let sel = if w % 4 == 3 { 1 } else { 0 };
    println!("sel={sel}");

    // n, s, e, w
    let corners = STEPS
        .iter()
        .map(|(dx, dy)| ((1 + dx) * half_dim, (1 + dy) * half_dim))
        .map(|p| fill(&grid, p, 2 * half_dim as usize)[0])
        .sum::<usize>();

    let m = half_dim * 2;
    let edges = [(0, 0), (0, m), (m, 0), (m, m)];

    // ne, nw, se, sw
    let outer_edges: usize = h * edges
        .iter()
        .map(|&p| fill(&grid, p, (half_dim - 1) as usize)[0])
        .sum::<usize>();

    // NE, NW, SE, SW
    let inner_edges: usize = (h - 1)
        * edges
            .iter()
            .map(|&p| fill(&grid, p, 3 * half_dim as usize)[1])
            .sum::<usize>();

    let blks = [h * h, (h - 1) * (h - 1)];
    let (blocks_even, blocks_odd) = (blks[1 - sel], blks[sel]);
    let fills = fill(&grid, start, (2 * grid_dim) as usize);
    let (d_even, d_odd) = (fills[1 - sel], fills[sel]);

    let dbg = cfg!(test) || crate::Cli::global().verbose;
    if dbg {
        println!("corners={corners} outer_edges={outer_edges} inner_edges={inner_edges}");
        println!("blocks_even={blocks_even} d_even={d_even}");
        println!("blocks_odd={blocks_odd} d_odd={d_odd}");
    }

    let inner_blocks = blocks_even * d_even + blocks_odd * d_odd;

    Ok(corners + outer_edges + inner_edges + inner_blocks)
}

fn fill(grid: &Grid<u8>, start: CellP, max_steps: usize) -> [usize; 2] {
    fill_ex(grid, start, max_steps).1
}

fn fill_ex(grid: &Grid<u8>, start: CellP, max_steps: usize) -> (Grid<Option<bool>>, [usize; 2]) {
    let mut vis = Grid::new(grid.dimensions(), None);

    *vis.get_mut(start).unwrap() = Some(false);

    let mut fifo = VecDeque::from([(0, start)]);
    let mut counts = [0, 0];
    while let Some((n, p)) = fifo.pop_front() {
        counts[n % 2] += 1;
        if n < max_steps {
            let n = n + 1;
            for &d in STEPS {
                let p = (p.0 + d.0, p.1 + d.1);
                if *grid.get(p).unwrap_or(&b'#') != b'#' {
                    let m = vis.get_mut(p).unwrap();
                    if m.is_none() {
                        *m = Some((n % 2) != 0);
                        fifo.push_back((n, p));
                    }
                }
            }
        }
    }

    let dbg = cfg!(test) || crate::Cli::global().verbose;
    if dbg {
        show_grid_vis(grid, &vis, "", Some(start));
    }

    (vis, counts)
}

fn show_grid_vis(grid: &Grid<u8>, vis: &Grid<Option<bool>>, prefix: &str, start: Option<CellP>) {
    let counts = vis.values().filter_map(|&x| x).fold([0, 0], |mut acc, b| {
        acc[b as usize] += 1;
        acc
    });
    println!(
        "{prefix}{}○:{} ●:{}",
        if prefix.is_empty() { "" } else { " " },
        counts[0],
        counts[1]
    );
    let rr = std::iter::zip(grid.rows(), vis.rows());
    for (y, (gr, vr)) in rr.enumerate() {
        let s = std::iter::zip(gr.iter(), vr.iter())
            .enumerate()
            .map(|(x, (&g, &v))| {
                if Some((x as i32, y as i32)) == start {
                    'S'
                } else {
                    match (g, v) {
                        (b'#', _) => '▒',
                        (_, Some(false)) => '○',
                        (_, Some(true)) => '●',
                        _ => '·',
                    }
                }
            })
            .collect::<String>();
        println!("{}{s}", if prefix.is_empty() { "" } else { "  " });
    }
    println!();
}

fn verify_problem(grid: &Grid<u8>, nsteps: usize) -> Result<CellP> {
    let (dx, dy) = grid.dimensions();

    if dx != dy {
        bail!("grid must be rectangular");
    }

    let start = grid.find(&b'S').unwrap();

    // start must be at center
    if start.0 * 2 + 1 != dx || start.1 * 2 + 1 != dy {
        bail!(
            "start point {:?} is not at the center with dimensions {:?}",
            start,
            (dx, dy)
        );
    }

    if (0..dx)
        .map(|x| grid.get((x, start.1)).unwrap())
        .any(|&c| c == b'#')
    {
        bail!("start row is not empty");
    }

    if (0..dy)
        .map(|y| grid.get((start.0, y)).unwrap())
        .any(|&c| c == b'#')
    {
        bail!("start column is not empty");
    }

    let x = start.0 as usize;
    if (nsteps - x).rem_euclid(dx as usize) != 0 {
        bail!("nsteps does not fall on a grid boundary");
    }

    Ok(start)
}

#[cfg(test)]
mod test {
    use super::*;

    fn calc_dumb(input: &str, nadd: usize, nsteps: usize) -> Result<usize> {
        let grid = Grid::parse(input)?;
        let (sx, sy) = grid.dimensions();

        let rpt_grid = repeat_grid(&grid, nadd);

        let (dx, dy) = rpt_grid.dimensions();
        let start = (dx / 2, dy / 2);

        if rpt_grid.get(start) != Some(&b'S') {
            bail!("start is not at the center");
        }

        let (vis, r) = fill_ex(&rpt_grid, start, nsteps);

        let m = nadd as i32;
        let mm = 2 * m + 1;

        let kind = |(x, y)| {
            let (dx, dy): (i32, i32) = (x - m, y - m);
            let n = dx.abs() + dy.abs();
            if n > m + 1 {
                // empty
                String::from("empty")
            } else if n == m + 1 {
                // outer edge
                String::from(["nw", "ne", "sw", "se"][quadrant((dx, dy))])
            } else if n == m {
                // corner or inner edge
                if dx == 0 {
                    // n/s corner
                    String::from(if dy < 0 { "n" } else { "s" })
                } else if dy == 0 {
                    // w/e corner
                    String::from(if dx < 0 { "w" } else { "e" })
                } else {
                    // inner edge
                    String::from(["NW", "NE", "SW", "SE"][quadrant((dx, dy))])
                }
            } else {
                // inner (even or odd)
                String::from(["even", "odd"][(n % 2) as usize])
            }
        };

        use std::collections::HashMap;
        let mut subgrids = HashMap::new();

        let subgrid_positions = (0..mm).flat_map(|y| (0..mm).map(move |x| (x, y)));
        for (x, y) in subgrid_positions {
            let k = kind((x, y));
            let topleft = (x * sx, y * sy);
            let sub = subgrid(&vis, topleft, (sx, sy));

            if let Some(last) = subgrids.get(&k) {
                if last != &sub {
                    println!("{k} mismatch at {x}, {y}");
                }
            } else {
                subgrids.insert(k, sub);
            }
        }

        for (k, sub) in subgrids.iter() {
            show_grid_vis(&grid, sub, k, None);
        }

        Ok(r[nsteps % 2])
    }

    // return index of [nw, ne, sw, se]
    fn quadrant(p: CellP) -> usize {
        if p.0 == 0 || p.1 == 0 {
            panic!("quadrant: invalid input");
        }
        let vx = if p.0 < 0 { 0 } else { 1 };
        let vy = if p.1 < 0 { 0 } else { 2 };
        vx + vy
    }

    fn subgrid<T>(grid: &Grid<T>, topleft: CellP, dim: CellP) -> Grid<T>
    where
        T: Clone,
    {
        let mut sub = Grid::new(dim, grid.get(topleft).unwrap().clone());
        let src_rows = grid.rows().skip(topleft.1 as usize).take(dim.1 as usize);
        for (sr, rr) in std::iter::zip(src_rows, sub.rows_mut()) {
            let sr = &sr[topleft.0 as usize..][..dim.0 as usize];
            rr.clone_from_slice(sr);
        }

        sub
    }

    fn repeat_grid(src: &Grid<u8>, nadd: usize) -> Grid<u8> {
        let (dx, dy) = src.dimensions();

        let m = 1 + 2 * (nadd as i32);
        let mut rpt = Grid::new((dx * m, dy * m), 0);

        let src = src.rows().map(|x| x.to_vec()).collect::<Vec<_>>();

        use std::iter::zip;
        for (sr, rr) in zip(src.iter().cycle(), rpt.rows_mut()) {
            for (sc, rc) in zip(sr.iter().cycle(), rr.iter_mut()) {
                *rc = *sc;
            }
        }

        rpt
    }

    fn run_checks(input: &str) {
        let grid = Grid::parse(input).expect("grid load failed");

        let dim = grid.dimensions().0 as usize;

        for nadd in 1..=5 {
            let nsteps = (nadd * 2 + 1) * dim / 2;
            println!("  {nsteps} steps");
            let dumb = calc_dumb(input, nadd, nsteps).expect("dumb calc failed");
            let smart = calc_smart(input, nsteps).expect("smart calc failed");

            assert_eq!(dumb, smart);
        }
    }

    #[test]
    fn it_works() {
        run_checks("...\n.S.\n...\n");

        let s2 = "\
...........
......##.#.
.###..#..#.
..#.#...#..
....#.#....
.....S.....
.##......#.
.......##..
.##.#.####.
.##...#.##.
...........
";
        run_checks(s2);
    }
}
