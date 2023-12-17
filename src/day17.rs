use crate::grid::{CellP, Grid};
use anyhow::Result;
use pathfinding::prelude::astar;

pub fn run(input: &str) -> Result<String> {
    let p1 = part1(input)?;
    let p2 = part2(input)?;
    Ok(format!("{p1} {p2}"))
}

fn part1(input: &str) -> Result<u32> {
    let g = Grid::parse(input)?;
    Ok(min_heat_loss(&g))
}

fn part2(_input: &str) -> Result<u32> {
    Ok(0)
}

fn min_heat_loss(grid: &Grid<u8>) -> u32 {
    let (dx, dy) = grid.dimensions();
    let goal = (dx - 1, dy - 1);
    let (v, c) = astar(
        &Node::pt(0, 0),
        |&n| n.successors(grid),
        |&n| n.distance(goal),
        |&n| n.pos == goal,
    )
    .unwrap();

    let dbg = cfg!(test) || crate::Cli::global().verbose;
    if dbg {
        let mut g2 = grid.clone();
        for (i, n) in v.iter().enumerate() {
            let c = b'a' + (i % 26) as u8;
            *g2.get_mut(n.pos).unwrap() = c;
        }
        g2.show();
    }

    c
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
struct Node {
    pos: CellP,

    // last step direction and count
    last_step_dir: CellP,
    last_step_count: usize,
}

impl Node {
    fn pt(x: i32, y: i32) -> Node {
        Node {
            pos: (x, y),
            ..Node::default()
        }
    }

    fn successors(&self, grid: &Grid<u8>) -> Vec<(Node, u32)> {
        const ADJ: &[CellP] = &[(-1, 0), (1, 0), (0, 1), (0, -1)];
        ADJ.iter()
            .filter(|&&d| d != back(self.last_step_dir))
            .filter_map(|&d| {
                let p = self.pos;
                let pos = (p.0 + d.0, p.1 + d.1);
                let last_step_count = if d == self.last_step_dir {
                    self.last_step_count + 1
                } else {
                    1
                };
                (grid.is_inside(pos) && last_step_count <= 3).then(|| {
                    (
                        Node {
                            pos,
                            last_step_dir: d,
                            last_step_count,
                        },
                        (grid.get(pos).unwrap() - b'0') as u32,
                    )
                })
            })
            .collect()
    }

    fn distance(&self, goal: CellP) -> u32 {
        self.pos.0.abs_diff(goal.0) + self.pos.1.abs_diff(goal.1)
    }
}

fn back(dir: CellP) -> CellP {
    (-dir.0, -dir.1)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let src = r"
2413432311323
3215453535623
3255245654254
3446585845452
4546657867536
1438598798454
4457876987766
3637877979653
4654967986887
4564679986453
1224686865563
2546548887735
4322674655533
";
        assert_eq!(part1(src).ok(), Some(102));
    }
}
