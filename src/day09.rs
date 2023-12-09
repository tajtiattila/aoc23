use anyhow::{anyhow, Result};

pub fn run(input: &str) -> Result<String> {
    Ok(format!("{}", p1(input)?))
}

fn p1(input: &str) -> Result<i64> {
    input
        .lines()
        .map(|line| {
            let r = line
                .split_whitespace()
                .map(|x| x.parse::<i64>().map_err(|_| anyhow!("invalid number: {x}")))
                .collect::<Result<Vec<_>, _>>();
            r.map(|v| extrapolate(v.iter().copied()))
        })
        .fold(Ok(0), |sum, r| match (sum, r) {
            (Ok(x), Ok(y)) => Ok(x + y),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        })
}

fn extrapolate(nums: impl Iterator<Item = i64>) -> i64 {
    let mut w = vec![nums.collect::<Vec<_>>()];

    loop {
        let mut it = w.last().unwrap().iter();
        let first = it.next().unwrap();
        let next: Vec<_> = it
            .scan(first, |state, n| {
                let x = n - *state;
                *state = n;
                Some(x)
            })
            .collect();

        if next.iter().all(|&x| x == 0) {
            break;
        }

        w.push(next);
    }

    w.iter().map(|v| v.last().unwrap_or(&0)).sum()
}
