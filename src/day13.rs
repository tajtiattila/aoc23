use anyhow::Result;
use std::iter::zip;

pub fn run(input: &str) -> Result<String> {
    let v = parse_valley(input);

    let p1 = v.iter().map(|m| m.reflection().unwrap()).sum::<usize>();
    let p2 = v.iter().map(|m| m.smudged().unwrap()).sum::<usize>();

    Ok(format!("{p1} {p2}"))
}

#[derive(Debug, Clone)]
struct Mirror(Vec<Vec<u8>>);

impl Mirror {
    #[allow(unused)]
    fn show(&self) {
        self.0
            .iter()
            .for_each(|l| println!("{}", unsafe { std::str::from_utf8_unchecked(l) }));
        println!();
    }

    fn reflection(&self) -> Option<usize> {
        self.reflections().next()
    }

    fn smudged(&self) -> Option<usize> {
        let org = self.reflection()?;

        let mut copy = self.clone();
        for y in 0..copy.0.len() {
            for x in 0..copy.0[y].len() {
                let oldc = copy.0[y][x];
                copy.0[y][x] = Self::flip(oldc);

                if let Some(x) = copy.single_refl_other_than(org) {
                    return Some(x);
                }

                copy.0[y][x] = oldc;
            }
        }

        None
    }

    fn single_refl_other_than(&self, other: usize) -> Option<usize> {
        let mut it = self.reflections().filter(|&x| x != other);
        let first = it.next()?;
        it.next().is_none().then_some(first)
    }

    fn flip(c: u8) -> u8 {
        match c {
            b'.' => b'#',
            b'#' => b'.',
            x => x,
        }
    }

    fn reflections(&self) -> impl Iterator<Item = usize> + '_ {
        self.refls_vert().chain(self.refls_horz())
    }

    fn refls_vert(&self) -> impl Iterator<Item = usize> + '_ {
        let my = self.0.len();
        (1..my)
            .filter(move |&y| zip((0..y).rev(), y..my).all(|(y0, y1)| self.0[y0] == self.0[y1]))
            .map(|v| 100 * v)
    }

    fn refls_horz(&self) -> impl Iterator<Item = usize> + '_ {
        let mx = self.0[0].len();
        (1..mx).filter(move |&x| zip((0..x).rev(), x..mx).all(|(x0, x1)| self.eq_cols(x0, x1)))
    }

    fn eq_cols(&self, x0: usize, x1: usize) -> bool {
        self.0.iter().all(|row| row[x0] == row[x1])
    }
}

fn parse_valley(input: &str) -> Vec<Mirror> {
    let mut mirrors = vec![];

    for line in input.lines() {
        if line.is_empty() {
            mirrors.push(Mirror(vec![]));
        } else {
            if mirrors.is_empty() {
                mirrors.push(Mirror(vec![]));
            }
            mirrors.last_mut().unwrap().0.push(line.as_bytes().to_vec());
        }
    }

    mirrors
}
