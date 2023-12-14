use anyhow::{bail, Result};

pub fn run(input: &str) -> Result<String> {
    Ok(format!("{}", p1(input)?))
}

fn p1(input: &str) -> Result<usize> {
    let mut p = Platform::load(input)?;
    p.roll(Dir::North);
    p.show();
    Ok(p.north_load())
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct Platform {
    dx: i32,
    dy: i32,
    m: Vec<u8>,
}

enum Dir {
    North,
    West,
    South,
    East,
}

impl Platform {
    fn load(input: &str) -> Result<Self> {
        let (dx, v) = input.lines().try_fold((0, vec![]), |(dx, mut v), line| {
            let b = line.as_bytes();
            if v.is_empty() {
                Ok((b.len(), b.to_vec()))
            } else {
                if b.len() != dx {
                    bail!("invalid line {line}");
                }
                v.extend_from_slice(b);
                Ok((dx, v))
            }
        })?;

        Ok(Self {
            dx: dx as i32,
            dy: (v.len() / dx) as i32,
            m: v,
        })
    }

    #[allow(unused)]
    fn show(&self) {
        self.m
            .chunks(self.dx as usize)
            .for_each(|row| println!("{}", String::from_utf8_lossy(row)));
    }

    fn roll(&mut self, dir: Dir) {
        match dir {
            Dir::North => self.roll_impl((0, 0), (0, 1), self.dy, (1, 0), self.dx),
            Dir::West => self.roll_impl((0, 0), (1, 0), self.dx, (0, 1), self.dy),
            Dir::South => self.roll_impl((0, self.dy - 1), (0, -1), self.dy, (1, 0), self.dx),
            Dir::East => self.roll_impl((self.dx - 1, 0), (-1, 0), self.dx, (0, 1), self.dy),
        }
    }

    fn north_load(&self) -> usize {
        self.m
            .chunks(self.dx as usize)
            .enumerate()
            .map(|(i, row)| ((self.dy as usize) - i) * row.iter().filter(|&&x| x == b'O').count())
            .sum()
    }

    fn roll_impl(
        &mut self,
        origin: (i32, i32),
        dstep: (i32, i32),
        nstep: i32,
        dslice: (i32, i32),
        nslice: i32,
    ) {
        let mut slc = origin;
        for _ in 0..nslice {
            self.roll_slice(slc, dstep, nstep);
            slc.0 += dslice.0;
            slc.1 += dslice.1;
        }
    }

    fn roll_slice(&mut self, origin: (i32, i32), dstep: (i32, i32), nstep: i32) {
        let mut p = origin.0 + origin.1 * self.dx;
        let dstep = dstep.0 + dstep.1 * self.dx;
        let mut free = p;
        for _ in 0..nstep {
            match self.m[p as usize] {
                b'#' => free = p + dstep,
                b'O' => {
                    if p != free {
                        self.m[free as usize] = b'O';
                        self.m[p as usize] = b'.';
                    }
                    free += dstep;
                }
                _ => {}
            }

            p += dstep;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let sample_src = "\
O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#....
";
        let sample_rolld_src = "\
OOOO.#.O..
OO..#....#
OO..O##..O
O..#.OO...
........#.
..#....#.#
..O..#.O.O
..O.......
#....###..
#....#....
";

        let mut sample = Platform::load(sample_src).unwrap();
        let sample_rolld = Platform::load(sample_rolld_src).unwrap();

        sample.roll(Dir::North);
        sample.show();
        assert_eq!(sample, sample_rolld);
    }
}
