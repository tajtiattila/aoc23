use anyhow::{anyhow, bail, Result};

pub fn run(input: &str) -> Result<String> {
    let p1 = part1(input)?;
    Ok(format!("{p1}"))
}

fn part1(input: &str) -> Result<usize> {
    let v = load_input(input)?;

    Ok(intersections_xy(
        &v,
        200_000_000_000_000,
        400_000_000_000_000,
    ))
}

fn load_input(input: &str) -> Result<Vec<Hailstone>> {
    input
        .lines()
        .map(Hailstone::from_str)
        .collect::<Result<Vec<_>>>()
}

type Coord = i64;
type Vec3 = (Coord, Coord, Coord);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Hailstone {
    p: Vec3,
    v: Vec3,
}

impl Hailstone {
    fn from_str(s: &str) -> Result<Self> {
        let w = s
            .split_whitespace()
            .enumerate()
            .filter(|(i, _)| i % 4 != 3)
            .map(|(_, n)| n.trim_end_matches(',').parse::<Coord>().ok())
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| anyhow!("invalid hailstone: {s}"))?;

        if w.len() != 6 {
            bail!("invalid hailstone: {s}");
        }

        let p = (w[0], w[1], w[2]);
        let v = (w[3], w[4], w[5]);
        Ok(Self { p, v })
    }
}

impl std::fmt::Display for Hailstone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} @ {}", vec3_strg(self.p), vec3_strg(self.v))
    }
}

fn intersections_xy(v: &[Hailstone], lo: i64, hi: i64) -> usize {
    let mut n = 0;
    for (i, a) in v.iter().enumerate() {
        for b in &v[(i + 1)..] {
            let ok = if let Some(c) = intersect_xy(*a, *b) {
                (lo..=hi).contains(&c.0) && (lo..=hi).contains(&c.1)
            } else {
                false
            };

            if ok {
                n += 1;
            }
        }
    }
    n
}

fn intersect_xy(a: Hailstone, b: Hailstone) -> Option<(Coord, Coord)> {
    let (x1, x2, x3, x4) = (a.p.0, a.p.0 + a.v.0, b.p.0, b.p.0 + b.v.0);
    let (y1, y2, y3, y4) = (a.p.1, a.p.1 + a.v.1, b.p.1, b.p.1 + b.v.1);

    let tn = (x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4);
    let td = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
    let un = (x1 - x3) * (y1 - y2) - (y1 - y3) * (x1 - x2);
    let ud = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);

    let nz = |n: i64, d: i64| d == 0 || n.signum() * d.signum() < 0;

    if nz(tn, td) || nz(un, ud) {
        return None;
    }

    let t = (tn as f64) / (td as f64);
    let u = (un as f64) / (ud as f64);

    let xt = (x1 as f64) + t * ((x2 - x1) as f64);
    let yt = (y1 as f64) + t * ((y2 - y1) as f64);

    let xu = (x3 as f64) + u * ((x4 - x3) as f64);
    let yu = (y3 as f64) + u * ((y4 - y3) as f64);

    let x = (xt + (xu - xt) / 2.0).round() as Coord; // (xt+xu)/2
    let y = (yt + (yu - yt) / 2.0).round() as Coord;

    Some((x, y))
}

fn vec3_strg(v: Vec3) -> String {
    format!("{:.3}, {:.3}, {:.3}", v.0, v.1, v.2)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        assert!(Hailstone::from_str("19, 13, 30 @ -2,  1, -2")
            .ok()
            .is_some());

        let sample = "\
19, 13, 30 @ -2,  1, -2
18, 19, 22 @ -1, -1, -2
20, 25, 34 @ -2, -2, -4
12, 31, 28 @ -1, -2, -1
20, 19, 15 @  1, -5, -3        
";
        let v = load_input(sample).unwrap();
        assert_eq!(intersections_xy(&v, 7, 27), 2);
    }
}
