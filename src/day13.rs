use anyhow::Result;
use std::iter::zip;

pub fn run(input: &str) -> Result<String> {
    let v = parse_valley(input);

    let p1 = v
        .iter()
        .map(|m| {
            m.show();
            m.reflection().unwrap()
        })
        .sum::<usize>();

    Ok(format!("{p1}"))
}

#[derive(Debug)]
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
        self.refl_vert().or_else(|| self.refl_horz())
    }

    fn refl_vert(&self) -> Option<usize> {
        let my = self.0.len();
        (1..my)
            .find(|&y| zip((0..y).rev(), y..my).all(|(y0, y1)| self.0[y0] == self.0[y1]))
            .map(|v| 100 * v)
    }

    fn refl_horz(&self) -> Option<usize> {
        let mx = self.0[0].len();
        (1..mx).find(|&x| zip((0..x).rev(), x..mx).all(|(x0, x1)| self.eq_cols(x0, x1)))
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
