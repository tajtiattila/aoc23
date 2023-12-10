use anyhow::{anyhow, bail, Result};

pub fn run(input: &str) -> Result<String> {
    let (p1, p2) = probl(input)?;

    Ok(format!("{p1} {p2}"))
}

fn probl(input: &str) -> Result<(usize, usize)> {
    let m = Map::parse(input)?;

    let l = m.find_loop().ok_or_else(|| anyhow!("loop not found"))?;

    Ok((l.len() / 2, m.count_enclosed(&l)))
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Pos2(i32, i32);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Dir {
    North,
    South,
    East,
    West,
}

impl Dir {
    fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
        }
    }
}

struct Map {
    dx: i32,
    dy: i32,
    m: Vec<u8>,
}

impl Map {
    fn parse(input: &str) -> Result<Self> {
        let (dx, m) = input
            .lines()
            .try_fold((0, Vec::new()), |(dx, mut v), line| {
                let bytes = line.as_bytes();
                if !v.is_empty() && dx != bytes.len() {
                    bail!("invalid line");
                }
                v.extend_from_slice(bytes);
                Ok((bytes.len(), v))
            })?;
        let dy = m.len() / dx;
        Ok(Map {
            dx: dx as i32,
            dy: dy as i32,
            m,
        })
    }

    fn start(&self) -> Option<Pos2> {
        self.m
            .iter()
            .position(|&c| c == b'S')
            .and_then(|p| self.to_xy(p))
    }

    fn find_loop(&self) -> Option<Vec<Pos2>> {
        let mut v = vec![self.start()?];
        let mut back_dir = None;
        const DIRS: &[Dir; 4] = &[Dir::North, Dir::South, Dir::West, Dir::East];
        loop {
            let &cur_pos = v.last().unwrap();
            let (next_pos, next_dir) = DIRS
                .iter()
                .filter(|&d| Some(*d) != back_dir)
                .find_map(|&d| self.step(cur_pos, d).map(|p| (p, d)))?;

            if Some(&next_pos) == v.first() {
                return Some(v);
            }

            v.push(next_pos);
            back_dir = Some(next_dir.opposite());
        }
    }

    fn count_enclosed(&self, pipe_loop: &[Pos2]) -> usize {
        const ON_LOOP: u8 = 1;
        const TO_NORTH: u8 = 2;
        let mut pipes = vec![0; self.m.len()];
        for &p in pipe_loop {
            pipes[self.to_pos(p).unwrap()] = ON_LOOP
                | if self.step(p, Dir::North).is_some() {
                    TO_NORTH
                } else {
                    0
                };
        }

        let mut inside = false;
        let mut enclosed = 0;

        let dbg = cfg!(test) || crate::Cli::global().verbose;

        for (i, &p) in pipes.iter().enumerate() {
            let mut is_enclosed = false;
            if p & TO_NORTH != 0 {
                inside = !inside;
            } else if inside && (p & ON_LOOP) == 0 {
                is_enclosed = true;
                enclosed += 1;
            }
            if dbg {
                if (p & ON_LOOP) != 0 {
                    print!("{}", Self::graphic(self.m[i]));
                } else if is_enclosed {
                    print!("■");
                } else {
                    print!("·");
                }
            }
            if dbg && (i + 1) % (self.dx as usize) == 0 {
                println!();
            }
        }

        enclosed
    }

    fn step(&self, p: Pos2, d: Dir) -> Option<Pos2> {
        let has = |px, py, what: fn(u8) -> bool| self.get(Pos2(px, py)).map(what).unwrap_or(false);

        let Pos2(px, py) = p;
        let check = |pipep: fn(u8) -> bool, qx, qy, pipeq: fn(u8) -> bool| {
            (has(px, py, pipep) && has(qx, qy, pipeq)).then_some(Pos2(qx, qy))
        };

        match d {
            Dir::North => check(Self::pipe_north, px, py - 1, Self::pipe_south),
            Dir::South => check(Self::pipe_south, px, py + 1, Self::pipe_north),
            Dir::West => check(Self::pipe_west, px - 1, py, Self::pipe_east),
            Dir::East => check(Self::pipe_east, px + 1, py, Self::pipe_west),
        }
    }

    fn to_xy(&self, p: usize) -> Option<Pos2> {
        (p < self.m.len()).then(|| {
            let i = p as i32;
            Pos2(i % self.dx, i / self.dx)
        })
    }

    fn to_pos(&self, p: Pos2) -> Option<usize> {
        self.is_inside(p).then(|| {
            let Pos2(px, py) = p;
            (px + py * self.dx) as usize
        })
    }

    fn is_inside(&self, p: Pos2) -> bool {
        let Pos2(px, py) = p;
        px >= 0 && px < self.dx && py >= 0 && py < self.dy
    }

    fn get(&self, p: Pos2) -> Option<u8> {
        self.to_pos(p).map(|p| self.m[p])
    }

    fn pipe_north(c: u8) -> bool {
        b"JL|S".contains(&c)
    }

    fn pipe_south(c: u8) -> bool {
        b"7F|S".contains(&c)
    }

    fn pipe_west(c: u8) -> bool {
        b"J7-S".contains(&c)
    }

    fn pipe_east(c: u8) -> bool {
        b"LF-S".contains(&c)
    }

    fn graphic(c: u8) -> char {
        match c {
            b'-' => '─',
            b'|' => '│',
            b'L' => '└',
            b'F' => '┌',
            b'7' => '┐',
            b'J' => '┘',
            b'.' => '·',
            x => x as char,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let count_inside = |src| {
            let m = Map::parse(src).ok()?;
            Some(m.count_enclosed(&m.find_loop()?))
        };

        assert_eq!(
            Some(4),
            count_inside(
                "\
...........
.S-------7.
.|F-----7|.
.||.....||.
.||.....||.
.|L-7.F-J|.
.|..|.|..|.
.L--J.L--J.
...........
"
            )
        );

        assert_eq!(
            Some(8),
            count_inside(
                "\
.F----7F7F7F7F-7....
.|F--7||||||||FJ....
.||.FJ||||||||L7....
FJL7L7LJLJ||LJ.L-7..
L--J.L7...LJS7F-7L7.
....F-J..F7FJ|L7L7L7
....L7.F7||L7|.L7L7|
.....|FJLJ|FJ|F7|.LJ
....FJL-7.||.||||...
....L---J.LJ.LJLJ...
"
            )
        );

        assert_eq!(
            Some(10),
            count_inside(
                "\
FF7FSF7F7F7F7F7F---7
L|LJ||||||||||||F--J
FL-7LJLJ||||||LJL-77
F--JF--7||LJLJ7F7FJ-
L---JF-JLJ.||-FJLJJ7
|F|F-JF---7F7-L7L|7|
|FFJF7L7F-JF7|JL---7
7-L-JL7||F7|L7F-7F7|
L.L7LFJ|||||FJL7||LJ
L7JLJL-JLJLJL--JLJ.L
"
            )
        );
    }
}
