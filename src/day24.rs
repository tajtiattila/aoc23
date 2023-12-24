use anyhow::{anyhow, bail, Result};
use fixed::types::I98F30;

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

type Coord = I98F30;
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

    fn pos_t(&self, t: Coord) -> Vec3 {
        let Self { p, v } = *self;
        (p.0 + v.0 * t, p.1 + v.1 * t, p.2 + v.2 * t)
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
            let (ok, c) = if let Some(c) = intersect_xy(*a, *b) {
                let ok = (lo..=hi).contains(&c.0) && (lo..=hi).contains(&c.1);
                (ok, Some(c))
            } else {
                (false, None)
            };

            if ok {
                n += 1;
            }
        }
    }
    n
}

fn collisions(v: &[Hailstone]) {
    for (i, a) in v.iter().enumerate() {
        for b in &v[(i + 1)..] {
            let s = if let Some(c) = intersect_xy(*a, *b) {
                format!("-> {}, {}", c.0, c.1)
            } else {
                "-> none".to_string()
            };

            println!("{a}; {b} {s}")
        }
    }
}

fn intersect_xy_0(a: Hailstone, b: Hailstone) -> Option<(Coord, (Coord, Coord))> {
    let tx = calc_t(a.p.0, a.v.0, b.p.0, b.v.0)?;
    let ty = calc_t(a.p.1, a.v.1, b.p.1, b.v.1)?;
    println!("  {tx} {ty}  {a}    {b}");

    let at = vec3_xy(a.pos_t(tx));
    let bt = vec3_xy(b.pos_t(tx));
    println!("  t={tx}:  {} {}; {} {}", at.0, at.1, bt.0, bt.1);
    eq2(at, bt).then_some((tx, at))
}

fn intersect_xy(a: Hailstone, b: Hailstone) -> Option<(Coord, Coord)> {
    let (x1, x2, x3, x4) = (a.p.0, a.p.0 + a.v.0, b.p.0, b.p.0 + b.v.0);
    let (y1, y2, y3, y4) = (a.p.1, a.p.1 + a.v.1, b.p.1, b.p.1 + b.v.1);

    let tn = (x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4);
    let td = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
    let t = tn.checked_div(td)?;

    let un = (x1 - x3) * (y1 - y2) - (y1 - y3) * (x1 - x2);
    let ud = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
    let u = un.checked_div(ud)?;

    if t < 0 || u < 0 {
        return None; // intersection in the past
    }

    let xt = x1 + t * (x2 - x1);
    let yt = y1 + t * (y2 - y1);

    let xu = x3 + u * (x4 - x3);
    let yu = y3 + u * (y4 - y3);

    let x = xt + (xu - xt) / 2; // (xt+xu)/2
    let y = yt + (yu - yt) / 2;

    Some((x, y))
}

fn calc_t(ap: Coord, av: Coord, bp: Coord, bv: Coord) -> Option<Coord> {
    // ap+av*t = bp+bv*t
    //   ap-bp = t*(bv-av)
    if ap == bp {
        Some(Coord::from_num(0))
    } else {
        Some((ap - bp) / (bv - av))
    }
}

fn eq2(a: (Coord, Coord), b: (Coord, Coord)) -> bool {
    let d0 = a.0.abs_diff(b.0);
    let d1 = a.1.abs_diff(b.1);
    println!("{d0} {d1}");
    d0 < 0.0001 && d1 < 0.0001
}

fn vec3_xy(v: Vec3) -> (Coord, Coord) {
    (v.0, v.1)
}

fn vec3_strg(v: Vec3) -> String {
    format!("{:.3}, {:.3}, {:.3}", v.0, v.1, v.2)
}

fn eq_dir2(a: (Coord, Coord), b: (Coord, Coord)) -> bool {
    // a.0/a.1 == b.0/b.1
    // a.0*b.1 == b.0*a.1
    a.0 * b.1 == b.0 * a.1
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
