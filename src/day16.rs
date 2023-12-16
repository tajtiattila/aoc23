use anyhow::{bail, Result};

pub fn run(input: &str) -> Result<String> {
    let p1 = part1(input)?;
    Ok(format!("{p1}"))
}

fn part1(input: &str) -> Result<usize> {
    let mut m = Map::parse(input)?;
    m.fire((0, 0), Dir::Right);
    Ok(m.count_energized())
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

struct Map {
    dx: i32,
    dy: i32,
    w: Vec<Tile>,
}

impl Map {
    fn parse(input: &str) -> Result<Map> {
        let (dx, w) = input
            .lines()
            .try_fold((0, vec![]), |(mut dx, mut w), line| {
                let v: Vec<_> = line.as_bytes().iter().map(|&c| Tile::from(c)).collect();
                if w.is_empty() {
                    dx = v.len();
                } else if v.len() != dx {
                    bail!("invalid line {line}");
                }
                w.extend_from_slice(&v);
                Ok((dx, w))
            })?;

        Ok(Self {
            dx: dx as i32,
            dy: (w.len() / dx) as i32,
            w,
        })
    }

    fn fire(&mut self, p: (i32, i32), dir: Dir) {
        let mut p = p;
        let mut dir = dir;

        loop {
            if !(0..self.dx).contains(&p.0) || !(0..self.dy).contains(&p.1) {
                return; // outside of the box
            }
            let pi = (p.0 + p.1 * self.dx) as usize;

            let dm = 1 << dir.idx();
            let tile = &mut self.w[pi];
            if tile.l & dm != 0 {
                return; // visited already
            }

            tile.l |= dm;

            let step = |d: Dir| {
                let d = d.delta();
                (p.0 + d.0, p.1 + d.1)
            };
            let next = |d| (step(d), d);

            let d = dir.delta();
            match tile.c {
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
                        self.fire(step(Dir::Up), Dir::Up);
                        (p, dir) = next(Dir::Down);
                    } else {
                        p = step(dir);
                    }
                }
                b'-' => {
                    if d.1 != 0 {
                        self.fire(step(Dir::Left), Dir::Left);
                        (p, dir) = next(Dir::Right);
                    } else {
                        p = step(dir);
                    }
                }
                _ => p = step(dir),
            }
        }
    }

    fn count_energized(&self) -> usize {
        self.w.iter().filter(|t| t.l != 0).count()
    }
}

#[derive(Debug, Copy, Clone)]
struct Tile {
    c: u8, // char from input
    l: u8, // light travel dir mask
}

impl Tile {
    fn from(c: u8) -> Self {
        Self { c, l: 0 }
    }
}
