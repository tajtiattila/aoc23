use anyhow::{anyhow, bail, Result};
use itertools::Interleave;

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

fn load_input(input: &str) -> Result<Vec<Stone>> {
    input
        .lines()
        .map(Stone::from_str)
        .collect::<Result<Vec<_>>>()
}

type Coord = i64;
type Vec3 = (Coord, Coord, Coord);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Stone {
    p: Vec3,
    v: Vec3,
}

impl Stone {
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
        let (p, v) = (self.p, self.v);
        (p.0 + v.0 * t, p.1 + v.1 * t, p.2 + v.2 * t)
    }

    fn x(&self) -> (Coord, Coord) {
        (self.p.0, self.v.0)
    }

    fn y(&self) -> (Coord, Coord) {
        (self.p.1, self.v.1)
    }

    fn z(&self) -> (Coord, Coord) {
        (self.p.2, self.v.2)
    }
}

impl std::fmt::Display for Stone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} @ {}", v3_strg(self.p), v3_strg(self.v))
    }
}

fn intersections_xy(v: &[Stone], lo: i64, hi: i64) -> usize {
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

fn intersect_xy_0(a: Stone, b: Stone) -> Option<(Coord, Coord)> {
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

    let x = (xt + xu / 2.0).round() as Coord;
    let y = (yt + yu / 2.0).round() as Coord;

    Some((x, y))
}

fn intersect_xy(a: Stone, b: Stone) -> Option<(Coord, Coord)> {
    let (in_past, t, u) = intersect_xy_tu(a, b)?;
    if in_past {
        return None;
    }

    let xt = (a.p.0 as f64) + t * (a.v.0 as f64);
    let yt = (a.p.1 as f64) + t * (a.v.1 as f64);

    let xu = (b.p.0 as f64) + u * (b.v.0 as f64);
    let yu = (b.p.1 as f64) + u * (b.v.1 as f64);

    let x = ((xt + xu) / 2.0).round() as Coord;
    let y = ((yt + yu) / 2.0).round() as Coord;

    Some((x, y))
}

fn collision_time(a: Stone, b: Stone) -> Option<Coord> {
    let (_, t, u) = intersect_xy_tu(a, b)?;
    let t = t.round() as Coord;
    let u = u.round() as Coord;

    (t == u && a.pos_t(t) == b.pos_t(u)).then_some(t)
}

fn intersect_xy_tu(a: Stone, b: Stone) -> Option<(bool, f64, f64)> {
    let (x1, x2, x3, x4) = (a.p.0, a.p.0 + a.v.0, b.p.0, b.p.0 + b.v.0);
    let (y1, y2, y3, y4) = (a.p.1, a.p.1 + a.v.1, b.p.1, b.p.1 + b.v.1);

    let tn = (x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4);
    let td = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
    let un = (x1 - x3) * (y1 - y2) - (y1 - y3) * (x1 - x2);
    let ud = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);

    if td == 0 || ud == 0 {
        return None;
    }

    let p = |n: i64, d: i64| n.signum() * d.signum() < 0;

    let in_past = p(tn, td) || p(un, ud);

    let t = (tn as f64) / (td as f64);
    let u = (un as f64) / (ud as f64);

    Some((in_past, t, u))
}

fn rock_throw(v: &[Stone]) -> usize {
    let (s0, s1) = (v[0], v[1]);

    const T_MAX: Coord = 1000;

    for t in 1..T_MAX {
        let p = s0.p;
        let q = s1.pos_t(t);

        let dt = v3_sub(q, p);
        if let Some(d) = v3_div_exact(dt, t) {
            println!("  {}; {} => {} {t}", v[0], v[1], v3_strg(d));
            if valid_rock_throw(v, p, d) {
                println!(" valid");
            }
        }
    }

    todo!();
    0
}

/*
fn rock_throw_dirs(v: &[Stone], t_max: Coord) -> usize {
    let (s0, s1) = (v[0], v[1]);

    let p = s0.p;

    (1..t_max).filter_map(|t| {
        let q = s1.pos_t(t);
        let dt = v3_sub(q, p);

    })
}
*/

fn valid_rock_throw(v: &[Stone], from: Vec3, dir: Vec3) -> bool {
    let rock = Stone { p: from, v: dir };
    v.iter().all(|&s| collision_time(rock, s).is_some())
}

fn v3_div_exact(a: Vec3, d: Coord) -> Option<Vec3> {
    (a.0 % d == 0 && a.1 % d == 0 && a.2 % d == 0).then(|| (a.0 / d, a.1 / d, a.2 / d))
}

fn v3_sub(a: Vec3, b: Vec3) -> Vec3 {
    (a.0 - b.0, a.1 - b.1, a.2 - b.2)
}

fn v3_strg(v: Vec3) -> String {
    format!("{:.3}, {:.3}, {:.3}", v.0, v.1, v.2)
}

fn calc_extrema(v: &[Stone]) {
    let calc = |n, f: fn(&Stone) -> (Coord, Coord)| {
        println!("{n}: {:?}", calc_min_max(v.iter().map(f)));
    };

    calc("x", Stone::x);
    calc("y", Stone::y);
    calc("z", Stone::z);
}

fn calc_min_max(it: impl Iterator<Item = (Coord, Coord)>) -> (Option<Coord>, Option<Coord>) {
    it.fold((None, None), |(mut acc_min, mut acc_max), (x, v)| {
        if v >= 0 {
            acc_min = Some(acc_min.unwrap_or(x).min(x));
        }
        if v <= 0 {
            acc_max = Some(acc_max.unwrap_or(x).max(x));
        }
        (acc_min, acc_max)
    })
}

fn init_times(times: &mut [Coord], w: &[(Coord, Coord)], v0: Coord) -> bool {
    for (t, (x, v)) in std::iter::zip(times.iter_mut(), w.iter()) {
        let v = v + v0;
        todo!()
    }

    true
}

fn find_rock(v: &[Stone]) -> Option<Stone> {
    const MAXC: i64 = 500;
    for d in xy_upto(MAXC) {
        if let Some(xy) = common_intersection_xy(v, d) {
            return Some(Stone {
                p: (xy.0, xy.1, 0),
                v: (d.0, d.1, 0),
            });
        }
    }

    None
}

fn common_intersection_xy(v: &[Stone], d: (Coord, Coord)) -> Option<(Coord, Coord)> {
    if v.len() < 4 {
        if v.len() < 2 {
            None
        } else {
            let rel = |s: Stone| Stone {
                p: s.p,
                v: v3_sub(s.v, (d.0, d.1, 0)),
            };

            let s0 = rel(v[0]);
            let s1 = rel(v[1]);
            let l = intersect_xy(s0, s1)?;
            if v.len() == 2 {
                return Some(l);
            }

            let s2 = rel(v[2]);
            let r = intersect_xy(s0, s2)?;

            (l == r).then_some(l)
        }
    } else {
        let i = v.len() / 2;
        let l = common_intersection_xy(&v[..i], d)?;
        let r = common_intersection_xy(&v[i..], d)?;
        (l == r).then_some(l)
    }
}

fn xy_upto(max: Coord) -> impl Iterator<Item = (Coord, Coord)> {
    let spiral =
        (1..max).flat_map(|u| (-u..u).flat_map(move |v| [(-u, v), (v, u), (u, -v), (-v, -u)]));
    std::iter::once((0, 0)).chain(spiral)
}

fn coords_upto(max: Coord) -> impl Iterator<Item = Coord> {
    let plusminus = (1..max).flat_map(|n| [-n, n].into_iter());
    std::iter::once(0).chain(plusminus)
}

#[cfg(test)]
mod test {
    use super::*;

    // xr+vr*t0 = x0+v0*t0
    // xr+vr*t1 = x1+v1*t1
    // xr+vr*t2 = x2+v2*t2
    // ...
    // xr+vr*tn = xn+vn*tn

    // -3:
    // 19@-2  19@+1
    // 18@-1  18@+2
    // 20@-2  20@+1
    // 12@-1  12@+2
    // 20@ 1  20@+4

    #[test]
    fn it_works() {
        assert!(Stone::from_str("19, 13, 30 @ -2,  1, -2").ok().is_some());

        let sample = "\
19, 13, 30 @ -2,  1, -2
18, 19, 22 @ -1, -1, -2
20, 25, 34 @ -2, -2, -4
12, 31, 28 @ -1, -2, -1
20, 19, 15 @  1, -5, -3        
";
        let v = load_input(sample).unwrap();
        assert_eq!(intersections_xy(&v, 7, 27), 2);

        println!("find_rock: {:?}", find_rock(&v));
        println!("ci xy = {:?}", common_intersection_xy(&v, (-3, 1)));
        println!("{:?}", xy_upto(5).collect::<Vec<_>>());

        calc_extrema(&v);

        assert!(valid_rock_throw(&v, (24, 13, 10), (-3, 1, 2)));
        rock_throw(&v);
    }
}
