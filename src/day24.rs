use anyhow::{anyhow, bail, Result};

pub fn run(input: &str) -> Result<String> {
    let v = load_input(input)?;
    let p1 = part1(&v);
    let p2 = part2(&v).ok_or_else(|| anyhow!("can't find rock throw"))?;
    Ok(format!("{p1} {p2}"))
}

fn part1(v: &[Stone]) -> usize {
    intersections_xy(&v, 200_000_000_000_000, 400_000_000_000_000)
}

fn part2(v: &[Stone]) -> Option<i64> {
    find_rock(v).map(|Stone { p, v: _ }| p.0 + p.1 + p.2)
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

fn v3_sub(a: Vec3, b: Vec3) -> Vec3 {
    (a.0 - b.0, a.1 - b.1, a.2 - b.2)
}

fn v3_strg(v: Vec3) -> String {
    format!("{:.3}, {:.3}, {:.3}", v.0, v.1, v.2)
}

fn find_rock(v: &[Stone]) -> Option<Stone> {
    const MAXC: i64 = 500;
    for d in xy_upto(MAXC) {
        let xy_rel = |s: &Stone| Stone {
            p: s.p,
            v: v3_sub(s.v, (d.0, d.1, 0)),
        };

        if let Some(xy) = common_intersection_2d(v, xy_rel) {
            for z in coord_upto(MAXC) {
                let d = (d.0, d.1, z);
                let xz_rel = |s: &Stone| {
                    let Stone { p, v } = *s;
                    let p = (p.0, p.2, p.1);
                    let v = (v.0 - d.0, v.2 - d.2, v.1 - d.1);

                    Stone { p, v }
                };

                if let Some(xz) = common_intersection_2d(v, xz_rel) {
                    return Some(Stone {
                        p: (xy.0, xy.1, xz.1),
                        v: d,
                    });
                }
            }
        }
    }

    None
}

fn common_intersection_2d<F>(v: &[Stone], mut f: F) -> Option<(Coord, Coord)>
where
    F: FnMut(&Stone) -> Stone + Copy,
{
    if v.len() < 4 {
        if v.len() < 2 {
            None
        } else {
            let s0 = f(&v[0]);
            let s1 = f(&v[1]);
            let l = intersect_xy(s0, s1)?;
            if v.len() == 2 {
                return Some(l);
            }

            let s2 = f(&v[2]);
            let r = intersect_xy(s0, s2)?;

            (l == r).then_some(l)
        }
    } else {
        let i = v.len() / 2;
        let l = common_intersection_2d(&v[..i], f)?;
        let r = common_intersection_2d(&v[i..], f)?;
        (l == r).then_some(l)
    }
}

fn xy_upto(max: Coord) -> impl Iterator<Item = (Coord, Coord)> {
    let spiral =
        (1..max).flat_map(|u| (-u..u).flat_map(move |v| [(-u, v), (v, u), (u, -v), (-v, -u)]));
    std::iter::once((0, 0)).chain(spiral)
}

fn coord_upto(max: Coord) -> impl Iterator<Item = Coord> {
    let plusminus = (1..max).flat_map(|n| [-n, n].into_iter());
    std::iter::once(0).chain(plusminus)
}

#[cfg(test)]
mod test {
    use super::*;

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

        assert_eq!(intersections_xy(&v, 7, 27,), 2);

        assert_eq!(
            find_rock(&v),
            Some(Stone {
                p: (24, 13, 10),
                v: (-3, 1, 2)
            })
        );
    }
}
