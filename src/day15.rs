use anyhow::{anyhow, Result};

pub fn run(input: &str) -> Result<String> {
    let p1 = input.trim().split(',').map(xhash).sum::<usize>();
    let p2 = part2(input);

    Ok(format!("{p1} {p2}"))
}

fn xhash(s: &str) -> usize {
    s.as_bytes()
        .iter()
        .fold(0, |acc, &c| ((acc + c as usize) * 17) % 256)
}

fn part2(input: &str) -> usize {
    let mut wall = Wall::new();

    for op in input.trim().split(',') {
        wall.handle(op).expect("invalid op");
    }

    let dbg = cfg!(test) || crate::Cli::global().verbose;
    if dbg {
        wall.show();
    }

    wall.power()
}

type TinyStr = tinystr::TinyAsciiStr<8>;

#[derive(Debug, Clone)]
struct Wall(Vec<LensBox>);

impl Wall {
    fn new() -> Self {
        Self(vec![LensBox::new(); 256])
    }

    fn handle(&mut self, op: &str) -> Result<()> {
        let i = op
            .find(['=', '-'])
            .ok_or_else(|| anyhow!("operator {op} is invalid"))?;
        let label = &op[..i];
        let box_idx = xhash(label);
        if op.as_bytes()[i] == b'=' {
            let focal_len = op[i + 1..].parse()?;
            self.0[box_idx].add(Lens::new(label, focal_len));
        } else {
            self.0[box_idx].remove(label);
        }

        Ok(())
    }

    fn power(&self) -> usize {
        self.0.iter().enumerate().map(|(i, b)| b.power(i)).sum()
    }

    fn show(&self) {
        for (i, b) in self.0.iter().enumerate().filter(|(_, b)| !b.0.is_empty()) {
            print!("Box {i}:");
            for l in &b.0 {
                print!(" [{} {}]", l.label, l.focal_len);
            }
            println!(" -> {}", b.power(i))
        }
    }
}

#[derive(Debug, Clone)]
struct LensBox(Vec<Lens>);

impl LensBox {
    fn new() -> Self {
        Self(vec![])
    }

    fn add(&mut self, l: Lens) {
        if let Some(i) = self.0.iter().position(|x| x.label == l.label) {
            self.0[i] = l;
        } else {
            self.0.push(l);
        }
    }

    fn remove(&mut self, l: &str) {
        self.0.retain(|x| x.label != l)
    }

    fn power(&self, box_index: usize) -> usize {
        (box_index + 1)
            * self
                .0
                .iter()
                .enumerate()
                .map(|(i, l)| (i + 1) * l.focal_len as usize)
                .sum::<usize>()
    }
}

#[derive(Debug, Copy, Clone)]
struct Lens {
    label: TinyStr,
    focal_len: u8,
}

impl Lens {
    fn new(label: &str, focal_len: u8) -> Self {
        Self {
            label: label.parse().expect("failed to parse"),
            focal_len,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let sample = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";
        assert_eq!(part2(sample), 145);
    }
}
