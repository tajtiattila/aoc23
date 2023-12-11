use anyhow::Result;

pub fn run(input: &str) -> Result<String> {
    let (p1, p2) = run_impl(input)?;

    Ok(format!("{p1} {p2}"))
}

fn run_impl(input: &str) -> Result<(i64, i64)> {
    Ok((dist_sum(input, 2), dist_sum(input, 1_000_000)))
}

fn dist_sum(input: &str, empty_size: i64) -> i64 {
    let gm: Vec<_> = input.lines().map(|line| line.as_bytes().to_vec()).collect();

    let dx = gm.iter().map(|row| row.len()).max().unwrap_or(0);

    use std::iter::zip;

    let mut empty_cols = vec![true; dx];
    for row in &gm {
        for (empty, &b) in zip(empty_cols.iter_mut(), row.iter()) {
            if b != b'.' {
                *empty = false;
            }
        }
    }

    let mut galaxies = vec![];

    let mut y = 0;
    for row in &gm {
        let mut empty_row = true;
        let mut x = 0;
        for (&empty_col, &b) in zip(empty_cols.iter(), row.iter()) {
            if b != b'.' {
                galaxies.push((x, y));
                empty_row = false;
            }

            x += if empty_col { empty_size } else { 1 }
        }

        y += if empty_row { empty_size } else { 1 }
    }

    let mut dist_sum = 0;
    for (i, &(px, py)) in galaxies.iter().enumerate() {
        for &(qx, qy) in galaxies.iter().skip(i + 1) {
            let dx: i64 = px - qx;
            let dy: i64 = py - qy;
            dist_sum += dx.abs() + dy.abs();
        }
    }

    dist_sum
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let sample = "\
...#......
.......#..
#.........
..........
......#...
.#........
.........#
..........
.......#..
#...#.....
";

        assert_eq!(dist_sum(sample, 2), 374);
        assert_eq!(dist_sum(sample, 10), 1030);
        assert_eq!(dist_sum(sample, 100), 8410);
    }
}
