use anyhow::{bail, Result};

pub fn run(input: &str) -> Result<String> {
    let p1 = part1(input)?;
    let p2 = part2(input)?;
    Ok(format!("{p1} {p2}"))
}

fn part1(input: &str) -> Result<usize> {
    let m = Map::parse(input)?;
    Ok(m.count_energized((0, 0), Dir::Right))
}

fn part2(input: &str) -> Result<usize> {
    let m = Map::parse(input)?;

    let v = (0..m.dx).flat_map(|x| [((x, 0), Dir::Down), ((x, m.dy - 1), Dir::Up)]);
    let h = (0..m.dy).flat_map(|y| [((0, y), Dir::Right), ((m.dx - 1, y), Dir::Left)]);
    Ok(v.chain(h)
        .map(|(p, d)| m.count_energized(p, d))
        .max()
        .unwrap())
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Dir {
    Left,
    Right,
    Up,
    Down,
}

impl Dir {
    fn idx(self) -> u8 {
        match self {
            Dir::Left => 0,
            Dir::Right => 1,
            Dir::Up => 2,
            Dir::Down => 3,
        }
    }

    fn delta(self) -> (i32, i32) {
        match self {
            Dir::Left => (-1, 0),
            Dir::Right => (1, 0),
            Dir::Up => (0, -1),
            Dir::Down => (0, 1),
        }
    }
}

#[derive(Debug, Clone)]
struct Map {
    dx: i32,
    dy: i32,
    w: Vec<u8>,
}

impl Map {
    fn parse(input: &str) -> Result<Map> {
        let (dx, w) = input
            .lines()
            .try_fold((0, vec![]), |(mut dx, mut w), line| {
                let v = line.as_bytes();
                if w.is_empty() {
                    dx = v.len();
                } else if v.len() != dx {
                    bail!("invalid line {line}");
                }
                w.extend_from_slice(v);
                Ok((dx, w))
            })?;

        Ok(Self {
            dx: dx as i32,
            dy: (w.len() / dx) as i32,
            w,
        })
    }

    fn count_energized(&self, p: (i32, i32), dir: Dir) -> usize {
        let mut lights = vec![0; self.w.len()];

        self.fire(&mut lights, p, dir);

        lights.iter().filter(|&l| *l != 0).count()
    }

    fn fire(&self, lights: &mut [u8], p: (i32, i32), dir: Dir) {
        let mut p = p;
        let mut dir = dir;

        loop {
            if !(0..self.dx).contains(&p.0) || !(0..self.dy).contains(&p.1) {
                return; // outside of the box
            }
            let p_idx = (p.0 + p.1 * self.dx) as usize;

            let dm = 1 << dir.idx();
            let light = &mut lights[p_idx];
            if *light & dm != 0 {
                return; // visited already
            }

            *light |= dm;

            let step = |d: Dir| {
                let d = d.delta();
                (p.0 + d.0, p.1 + d.1)
            };
            let next = |d| (step(d), d);

            let d = dir.delta();
            match self.w[p_idx] {
                b'/' => {
                    (p, dir) = match dir {
                        Dir::Left => next(Dir::Down),
                        Dir::Right => next(Dir::Up),
                        Dir::Up => next(Dir::Right),
                        Dir::Down => next(Dir::Left),
                    }
                }
                b'\\' => {
                    (p, dir) = match dir {
                        Dir::Left => next(Dir::Up),
                        Dir::Right => next(Dir::Down),
                        Dir::Up => next(Dir::Left),
                        Dir::Down => next(Dir::Right),
                    }
                }
                b'|' => {
                    if d.0 != 0 {
                        self.fire(lights, step(Dir::Up), Dir::Up);
                        (p, dir) = next(Dir::Down);
                    } else {
                        p = step(dir);
                    }
                }
                b'-' => {
                    if d.1 != 0 {
                        self.fire(lights, step(Dir::Left), Dir::Left);
                        (p, dir) = next(Dir::Right);
                    } else {
                        p = step(dir);
                    }
                }
                _ => p = step(dir),
            }
        }
    }
}
