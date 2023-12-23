use anyhow::{anyhow, bail, Result};

use num::Integer;

pub fn run(input: &str) -> Result<String> {
    Ok(format!("{} {}", p1(input)?, p2(input)?))
}

fn p1(input: &str) -> Result<i64> {
    let nums = input
        .lines()
        .map(|line| {
            line.split_whitespace()
                .skip(1)
                .map(|num| num.parse().map_err(|_| anyhow!("error parsing {num}")))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>>>()?;

    if nums.len() != 2 {
        bail!("invalid input");
    }

    Ok(std::iter::zip(nums[0].iter(), nums[1].iter())
        .map(|(&t, &d)| num_wins(t, d))
        .product())
}

fn p2(input: &str) -> Result<i64> {
    let nums = input
        .lines()
        .map(|line| -> Result<_, _> {
            line.split_whitespace()
                .skip(1)
                .flat_map(|num| num.chars())
                .collect::<String>()
                .parse()
                .map_err(|_| anyhow!("error parsing {line}"))
        })
        .collect::<Result<Vec<_>>>()?;

    if nums.len() != 2 {
        bail!("invalid input");
    }

    Ok(num_wins(nums[0], nums[1]))
}

fn num_wins(t_max: i64, d_rec: i64) -> i64 {
    // Best hold time is where t yields better distance than t+1.
    let t_best = partition_point(1..t_max, |&t1| dist(t1 + 1, t_max) > dist(t1, t_max));

    // Find lowest and highest winning times.
    let t_lo = partition_point(1..t_best, |&t| dist(t, t_max) <= d_rec);
    let t_hi = partition_point(t_best..t_max, |&t| dist(t, t_max) > d_rec);

    t_hi - t_lo
}

fn dist(t_hold: i64, t_max: i64) -> i64 {
    let spd = t_hold;
    let t_travel = t_max - t_hold;
    spd * t_travel
}

fn partition_point<T, P>(rng: std::ops::Range<T>, mut pred: P) -> T
where
    P: FnMut(&T) -> bool,
    T: Integer + Copy,
{
    let one: T = T::one();
    let two: T = one + one;
    let (mut lo, mut hi) = (rng.start, rng.end);
    while lo < hi {
        let siz = hi - lo;
        let m = lo + siz / two;

        if pred(&m) {
            lo = m + one;
        } else {
            hi = m;
        }
    }

    assert!(lo == rng.start || pred(&(lo - one)));
    assert!(lo == rng.end || !pred(&hi));

    lo
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn num_wins_works() {
        let sample = "\
Time:      7  15   30
Distance:  9  40  200
";
        assert_eq!(p1(sample).ok(), Some(288));
        assert_eq!(p2(sample).ok(), Some(71503));
    }
}
