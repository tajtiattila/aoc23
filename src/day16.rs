use crate::grid::{CellP, Dir, Grid};
use anyhow::Result;

pub fn run(input: &str) -> Result<String> {
    let p1 = part1(input)?;
    let p2 = part2(input)?;
    Ok(format!("{p1} {p2}"))
}

fn part1(input: &str) -> Result<usize> {
    let g = Grid::parse(input)?;
    Ok(count_energized(&g, (0, 0), (1, 0)))
}

fn part2(input: &str) -> Result<usize> {
    let g = Grid::parse(input)?;

    let (dx, dy) = g.dimensions();
    let v = (0..dx).flat_map(|x| [((x, 0), (0, 1)), ((x, dy - 1), (0, -1))]);
    let h = (0..dy).flat_map(|y| [((0, y), (1, 0)), ((dx - 1, y), (-1, 0))]);
    Ok(v.chain(h)
        .map(|(p, d)| count_energized(&g, p, d))
        .max()
        .unwrap())
}

fn count_energized(grid: &Grid<u8>, p: CellP, dir: CellP) -> usize {
    let mut lights = Grid::new(grid.dimensions(), 0);

    fire(grid, &mut lights, p, dir);

    let dbg = cfg!(test) || crate::Cli::global().verbose;
    if dbg {
        println!("{};{}:", p.0, p.1);
        for r in lights.rows() {
            let s = r
                .iter()
                .map(|&x| if x > 0 { '#' } else { '·' })
                .collect::<String>();
            println!("  {s}");
        }
    }

    lights.values().filter(|&l| *l != 0).count()
}

fn fire(grid: &Grid<u8>, lights: &mut Grid<u8>, mut p: CellP, mut d: CellP) {
    loop {
        if !grid.is_inside(p) {
            return;
        }

        // mask corresponding the light direction
        let light_mask = 1 << Dir::from_xy(d).unwrap().index();

        let light = lights.get_mut(p).unwrap();
        if *light & light_mask != 0 {
            return; // visited already
        }
        *light |= light_mask;

        let step = |d0, d1| {
            let q = (p.0 + d0, p.1 + d1);
            (q, (d0, d1))
        };

        (p, d) = match grid.get(p).unwrap() {
            b'/' => step(-d.1, -d.0),
            b'\\' => step(d.1, d.0),
            b'|' => {
                if d.0 != 0 {
                    let (px, dx) = step(0, -1); // up
                    fire(grid, lights, px, dx);
                    step(0, 1) // down
                } else {
                    step(d.0, d.1) // no change
                }
            }
            b'-' => {
                if d.1 != 0 {
                    let (px, dx) = step(-1, 0); // left
                    fire(grid, lights, px, dx);
                    step(1, 0) // right
                } else {
                    step(d.0, d.1) // no change
                }
            }
            _ => step(d.0, d.1),
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let src = r"
.|...\....
|.-.\.....
.....|-...
........|.
..........
.........\
..../.\\..
.-.-/..|..
.|....-|.\
..//.|....
";
        assert_eq!(part1(src).ok(), Some(46));
        assert_eq!(part2(src).ok(), Some(51));
    }
}
