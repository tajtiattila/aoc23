use anyhow::{anyhow, bail, Context, Result};

pub fn run(input: &str) -> Result<String> {
    let alm = parse_almanac(input)?;

    Ok(format!("{} {}", p1(&alm)?, p2(&alm)?))
}

type Seed = i64;

fn p1(alm: &Almanac) -> Result<Seed> {
    alm.locations()
        .min()
        .ok_or_else(|| anyhow!("empty almanac"))
}

fn p2(alm: &Almanac) -> Result<Seed> {
    let pairs = alm
        .seeds
        .chunks(2)
        .map(|c| (c.len() == 2).then(|| (c[0], c[1])))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| anyhow!("odd seed pair in almanac"))?;

    let mut r: Option<Seed> = None;

    let mut have = |loc| {
        if r.is_none() || loc < r.unwrap() {
            r = Some(loc);
        }
    };

    for (lo, num) in pairs {
        let hi = lo + num;
        have(alm.map.apply(lo));

        let v = &alm.map.0;
        let i = v.partition_point(|e| e.src.1 <= lo);
        v[i..]
            .iter()
            .take_while(|m| m.src.0 < hi)
            .for_each(|m| have(m.dst.0));
    }

    r.ok_or_else(|| anyhow!("no seeds"))
}

struct Almanac {
    seeds: Vec<Seed>,
    map: Map,
}

impl Almanac {
    fn locations(&self) -> impl Iterator<Item = Seed> + '_ {
        self.seeds.iter().map(|&seed| self.map.apply(seed))
    }
}

#[derive(Clone)]
struct Map(Vec<MapEntry>);

impl Map {
    fn new() -> Self {
        Self(vec![])
    }

    fn apply(&self, seed: Seed) -> Seed {
        let i = self.0.partition_point(|x| x.src.0 <= seed);
        if i == 0 {
            return seed;
        }
        let m = &self.0[i - 1];
        m.apply(seed)
    }

    fn combine(&self, other: &Self) -> Self {
        let mut v0 = self.0.clone();
        v0.sort_by_key(|e| e.dst.0);

        let mut v1 = other.0.clone();
        v1.sort_by_key(|e| e.src.0);

        let mut r = vec![];
        let mut v0 = v0.as_mut_slice();
        let mut v1 = v1.as_mut_slice();
        while let (Some(f0), Some(f1)) = (v0.first_mut(), v1.first_mut()) {
            let (f0lo, f0hi) = f0.dst;
            let (f1lo, f1hi) = f1.src;
            if f0hi <= f1lo {
                // handle f0lo..f0hi
                r.push(*f0);
                v0 = &mut v0[1..];
            } else if f1hi <= f0lo {
                // handle f1lo..f1hi
                r.push(*f1);
                v1 = &mut v1[1..];
            } else if f0lo < f1lo {
                // handle f0lo..f1lo
                let d = f1lo - f0lo;
                r.push(f0.take(d));
            } else if f1lo < f0lo {
                // handle f1lo..f0lo
                let d = f0lo - f1lo;
                r.push(f1.take(d));
            } else {
                // f0lo == f1lo
                type O = std::cmp::Ordering;
                match f0hi.cmp(&f1hi) {
                    O::Less => {
                        let f1x = f1.take(f0.size());
                        r.push(MapEntry::new(f0.src, f1x.dst));
                        v0 = &mut v0[1..];
                    }
                    O::Equal => {
                        r.push(MapEntry::new(f0.src, f1.dst));
                        v0 = &mut v0[1..];
                        v1 = &mut v1[1..];
                    }
                    O::Greater => {
                        let f0x = f0.take(f1.size());
                        r.push(MapEntry::new(f0x.src, f1.dst));
                        v1 = &mut v1[1..];
                    }
                }
            }
        }

        r.extend_from_slice(v0);
        r.extend_from_slice(v1);

        r.sort_by_key(|e| e.src.0);

        Self(r)
    }
}

#[derive(Debug, Copy, Clone)]
struct MapEntry {
    src: (Seed, Seed),
    dst: (Seed, Seed),
}

impl MapEntry {
    fn new(src: (Seed, Seed), dst: (Seed, Seed)) -> Self {
        assert_eq!(src.1 - src.0, dst.1 - dst.0);
        Self { src, dst }
    }

    fn size(&self) -> Seed {
        self.src.1 - self.src.0
    }

    fn take(&mut self, d: Seed) -> Self {
        assert!(self.src.0 + d <= self.src.1);
        let r = Self {
            src: (self.src.0, self.src.0 + d),
            dst: (self.dst.0, self.dst.0 + d),
        };

        self.src.0 += d;
        self.dst.0 += d;

        r
    }

    fn apply(&self, n: Seed) -> Seed {
        if (self.src.0..self.src.1).contains(&n) {
            n - self.src.0 + self.dst.0
        } else {
            n
        }
    }
}

fn parse_almanac(input: &str) -> Result<Almanac> {
    let mut seeds = vec![];

    let mut combined_map = Map::new();
    let mut map = Map::new();

    for (ln, line) in input.lines().enumerate() {
        let ln = ln + 1;
        let line = line.trim();
        if let Some(nums) = line.strip_prefix("seeds:") {
            seeds = nums
                .split_whitespace()
                .map(|v| v.parse())
                .collect::<Result<Vec<_>, _>>()
                .with_context(|| format!("in line {ln}: {line}"))?;
        } else if let Some(mx) = line.strip_suffix(" map:") {
            let _ = mx
                .split_once("-to-")
                .ok_or_else(|| anyhow!("invalid map line {ln}: {line}"))?;
            combined_map = combined_map.combine(&map);
            map = Map::new();
        } else if !line.is_empty() {
            let v = line
                .split_whitespace()
                .map(|v| v.parse())
                .collect::<Result<Vec<_>, _>>()
                .with_context(|| format!("in line {ln}: {line}"))?;
            if v.len() != 3 {
                bail!("invalid map line {ln}: {line}");
            }
            let (d0, s0, l) = (v[0], v[1], v[2]);
            map.0.push(MapEntry {
                src: (s0, s0 + l),
                dst: (d0, d0 + l),
            })
        }
    }
    combined_map = combined_map.combine(&map);

    Ok(Almanac {
        seeds,
        map: combined_map,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn seed_to_loc() {
        let sample = "\
seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15

fertilizer-to-water map:
49 53 8
0 11 42
42 0 7
57 7 4

water-to-light map:
88 18 7
18 25 70

light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:
0 69 1
1 0 69

humidity-to-location map:
60 56 37
56 93 4
";
        let r = parse_almanac(sample);
        if let Err(ref err) = r {
            println!("{err}");
        }
        assert!(r.is_ok());

        let alm = r.unwrap();
        assert_eq!(alm.seeds.len(), 4);

        assert_eq!(p1(&alm).ok(), Some(35));

        assert_eq!(alm.map.apply(79), 82);
        assert_eq!(alm.map.apply(14), 43);
        assert_eq!(alm.map.apply(55), 86);
        assert_eq!(alm.map.apply(13), 35);

        assert_eq!(p2(&alm).ok(), Some(46));
    }
}
