use anyhow::Result;

pub fn run(input: &str) -> Result<String> {
    Ok(format!("{}", p1(input)))
}

fn p1(input: &str) -> usize {
    let platf = input
        .lines()
        .map(|l| l.as_bytes().to_vec())
        .collect::<Vec<_>>();

    let dx = platf[0].len();
    (0..dx)
        .map(|col| total_load(platf.len(), platf.iter().map(|row| row[col])))
        .sum()
}

fn total_load(size: usize, tiles: impl Iterator<Item = u8>) -> usize {
    tiles
        .enumerate()
        .fold(LoadAcc::new(size), |mut acc, (i, c)| {
            acc.add(i, c);
            acc
        })
        .total()
}

struct LoadAcc {
    size: usize,

    // last range
    first_free: usize,
    n_rocks: usize,

    loads_so_far: usize,
}

impl LoadAcc {
    fn new(size: usize) -> Self {
        Self {
            size,
            first_free: 0,
            n_rocks: 0,
            loads_so_far: 0,
        }
    }

    fn add(&mut self, i: usize, c: u8) {
        match c {
            b'O' => self.n_rocks += 1,
            b'#' => {
                self.loads_so_far += self.sum_last_range();
                self.first_free = i + 1;
                self.n_rocks = 0;
            }
            _ => {}
        }
    }

    fn sum_last_range(&self) -> usize {
        if self.n_rocks == 0 {
            return 0;
        }

        let hi = self.size - self.first_free;
        let lo = hi - self.n_rocks + 1;
        (hi + lo) * self.n_rocks / 2
    }

    fn total(&self) -> usize {
        self.loads_so_far + self.sum_last_range()
    }
}
