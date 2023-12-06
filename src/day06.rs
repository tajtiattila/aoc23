use anyhow::{anyhow, bail, Result};

pub fn run(input: &str) -> Result<String> {
    Ok(format!("{} {}", p1(input)?, p2(input)?))
}

fn p1(input: &str) -> Result<i64> {
    let nums = input
        .lines()
        .map(|line| {
            line.split_whitespace()
                .skip(1)
                .map(|num| num.parse())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| anyhow!("error parsing line {line}"))
        })
        .collect::<Result<Vec<_>>>()?;

    proc(&nums)
}

fn p2(_input: &str) -> Result<i64> {
    Ok(0)
}

fn proc(nums: &[Vec<i64>]) -> Result<i64> {
    if nums.len() != 2 {
        bail!("invalid input");
    }

    Ok(std::iter::zip(nums[0].iter(), nums[1].iter())
        .map(|(&t_max, &d_rec)| {
            println!("t_max={t_max} d_rec={d_rec}");
            (1..t_max)
                .filter(|t_hold| {
                    let spd = t_hold;
                    let t_travel = t_max - t_hold;
                    let d = spd * t_travel;
                    d > d_rec
                })
                .count() as i64
        })
        .product())
}

fn num_wins(t_max: i64, d_rec: i64) -> i64 {}

fn partition_point<T, P>(rng: std::ops::Range<T>, pred: P) -> T
where
    P: FnMut(&T) -> bool,
    T: PartialOrd
        + From<u8>
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + std::ops::Div<Output = T>,
{
    let (mut lo, mut hi) = (rng.start, rng.end);
    while lo + 1.into() < hi {
        let siz = hi - lo;
        let m = lo + siz / 2.into();

        if pred(&m) {
            lo = m + 1.into();
        } else {
            hi = m;
        }
    }

    lo
}
