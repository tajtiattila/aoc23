use anyhow::anyhow;

pub fn run(input: &str) -> anyhow::Result<String> {
    let p1 = run_calibr(input, calibr_1)?;
    let p2 = run_calibr(input, calibr_2)?;

    // 2: 52136 too low
    Ok(format!("{p1} {p2}"))
}

fn run_calibr(input: &str, f: impl FnMut(&str) -> anyhow::Result<u32>) -> anyhow::Result<u32> {
    input
        .lines()
        .map(f)
        .fold(Ok(0), |rsum, elem| match (rsum, elem) {
            (Ok(sum), Ok(e)) => Ok(sum + e),
            (Err(err), _) => Err(err),
            (_, Err(err)) => Err(err),
        })
}

fn calibr_1(line: &str) -> anyhow::Result<u32> {
    let x = line.chars().fold((None, None), |(f, l), c| {
        if let Some(d) = c.to_digit(10) {
            (f.or(Some(d)), Some(d))
        } else {
            (f, l)
        }
    });

    if let (Some(f), Some(l)) = x {
        Ok(f * 10 + l)
    } else {
        Err(anyhow!("Missing digits in string {}", line))
    }
}

fn calibr_2(line: &str) -> anyhow::Result<u32> {
    let digit_values = [
        ("0", 0),
        ("1", 1),
        ("one", 1),
        ("2", 2),
        ("two", 2),
        ("3", 3),
        ("three", 3),
        ("4", 4),
        ("four", 4),
        ("5", 5),
        ("five", 5),
        ("6", 6),
        ("six", 6),
        ("7", 7),
        ("seven", 7),
        ("8", 8),
        ("eight", 8),
        ("9", 9),
        ("nine", 9),
    ];

    let mut fst: Option<(u32, usize)> = None;
    let mut lst: Option<(u32, usize)> = None;
    for &(s, v) in &digit_values {
        if let Some(p) = line.find(s) {
            if fst.is_none() || p < fst.unwrap().1 {
                fst = Some((v, p))
            }
        }
        if let Some(p) = line.rfind(s) {
            if lst.is_none() || p > lst.unwrap().1 {
                lst = Some((v, p))
            }
        }
    }

    if let (Some((f, _)), Some((l, _))) = (fst, lst) {
        Ok(f * 10 + l)
    } else {
        Err(anyhow!("Missing digits in string {}", line))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn day01_test() {
        let sample = "\
two1nine
eightwothree
abcone2threexyz
xtwone3four
4nineeightseven2
zoneight234
7pqrstsixteen";
        assert_eq!(run_calibr(sample, calibr_2).unwrap(), 281);
    }
}
