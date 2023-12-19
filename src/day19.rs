use std::{cmp::Ordering, collections::HashMap};

use anyhow::{anyhow, Result};

pub fn run(input: &str) -> Result<String> {
    Ok(format!("{} {}", part1(input)?, part2(input)?))
}

fn part1(input: &str) -> Result<usize> {
    let (plan, parts) = Plan::load(input)?;

    Ok(parts
        .iter()
        .filter(|part| plan.is_accepted(part))
        .map(|part| part.sum_rating())
        .sum())
}

fn part2(input: &str) -> Result<usize> {
    let (plan, _) = Plan::load(input)?;

    Ok(plan.count_accepted(1, 4000))
}

type TinyStr = tinystr::TinyAsciiStr<4>;
type Rating = u16;

#[allow(unused)]
struct Plan {
    wf: Vec<Workflow>,

    wf_index: HashMap<TinyStr, usize>,

    start_index: usize,
}

impl Plan {
    fn load(input: &str) -> Result<(Plan, Vec<Part>)> {
        let wf_index = input
            .lines()
            .take_while(|l| !l.is_empty())
            .enumerate()
            .map(|(ix, line)| {
                let (label, _) = line
                    .split_once('{')
                    .ok_or_else(|| anyhow!("missing '{{' in line {}: {}", ix + 1, line))?;
                let lp = label
                    .parse::<TinyStr>()
                    .map_err(|_| anyhow!("invalid label {} in line {}: {}", label, ix + 1, line));
                Ok((lp?, ix))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        let start_index = *wf_index
            .get(&"in".parse::<TinyStr>().unwrap())
            .ok_or_else(|| anyhow!("'in' rule missing"))?;

        let line_workflow = |line: &str| -> Result<Workflow> {
            let mut it = line.split(['{', ',', '}']).filter(|s| !s.is_empty());

            let label = it.next().and_then(|s| s.parse::<TinyStr>().ok()).unwrap();
            let rules = it
                .map(|s| {
                    let srule = Rule::parse(s)?;
                    let ix = srule.target.map_workflow(|s| wf_index.get(&s).copied())?;
                    Some(Rule {
                        cond: srule.cond,
                        target: ix,
                    })
                })
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| anyhow!("invalid rule in line {line}"))?;

            Ok(Workflow { label, rules })
        };

        let line_part =
            |line: &str| Part::parse(line).ok_or_else(|| anyhow!("invalid part in line {line}"));

        let mut wf = vec![];
        let mut parts = vec![];
        let mut in_parts = false;
        for line in input.lines() {
            if line.is_empty() {
                in_parts = true;
            } else if in_parts {
                parts.push(line_part(line)?);
            } else {
                wf.push(line_workflow(line)?);
            }
        }

        Ok((
            Plan {
                wf,
                wf_index,
                start_index,
            },
            parts,
        ))
    }

    fn is_accepted(&self, p: &Part) -> bool {
        self.is_accepted_impl(p) == Some(true)
    }

    fn is_accepted_impl(&self, p: &Part) -> Option<bool> {
        let mut i = self.start_index;
        loop {
            let wf = &self.wf[i];
            match wf.rules.iter().find_map(|r| r.consider(p))? {
                Target::Accept => return Some(true),
                Target::Reject => return Some(false),
                Target::Workflow(j) => i = *j,
            }
        }
    }

    fn count_accepted(&self, lo: Rating, hi: Rating) -> usize {
        let mut c = CountAccepted::new(self);
        c.count(self.start_index, PartSpace::range(lo, hi))
    }
}

struct CountAccepted<'a> {
    plan: &'a Plan,
    dbg: bool,
}

impl<'a> CountAccepted<'a> {
    fn new(plan: &'a Plan) -> Self {
        Self {
            plan,

            dbg: cfg!(test) || crate::Cli::global().verbose,
        }
    }

    fn count(&mut self, i: usize, ps: PartSpace) -> usize {
        if self.dbg {
            println!("{}: {ps}", self.plan.wf[i].label);
        }

        self.plan.wf[i]
            .rules
            .iter()
            .fold((0, ps), |(sum, acc), rule| {
                let (add, next_ps) = if let Some(c) = &rule.cond {
                    let (l, r) = c.split(&acc);
                    (self.handle(&rule.target, l), r)
                } else {
                    (self.handle(&rule.target, acc), PartSpace::empty())
                };
                (sum + add, next_ps)
            })
            .0
    }

    fn handle(&mut self, target: &Target<usize>, ps: PartSpace) -> usize {
        if self.dbg {
            println!(" {ps}");
        }

        match target {
            Target::Accept => ps.total_count(),
            Target::Reject => 0,
            Target::Workflow(j) => {
                if !ps.is_empty() {
                    self.count(*j, ps)
                } else {
                    0
                }
            }
        }
    }
}

#[allow(unused)]
struct Workflow {
    label: TinyStr,

    rules: Vec<Rule<usize>>,
}

struct Rule<T> {
    cond: Option<Condition>,
    target: Target<T>, // workflow index
}

impl Rule<TinyStr> {
    fn parse(def: &str) -> Option<Self> {
        if let Some((l, r)) = def.split_once(':') {
            let cond = Some(Condition::parse(l)?);
            let target = Target::parse(r)?;
            Some(Self { cond, target })
        } else {
            Target::parse(def).map(|target| Self { cond: None, target })
        }
    }
}

impl<T> Rule<T> {
    fn consider<'a>(&'a self, part: &Part) -> Option<&'a Target<T>> {
        match &self.cond {
            Some(c) if !c.match_part(part) => None,
            _ => Some(&self.target),
        }
    }
}

enum Target<T> {
    Accept,
    Reject,
    Workflow(T),
}

impl Target<TinyStr> {
    fn parse(def: &str) -> Option<Self> {
        match def {
            "A" => Some(Self::Accept),
            "R" => Some(Self::Reject),
            _ => def.parse::<TinyStr>().ok().map(Self::Workflow),
        }
    }
}

impl<T> Target<T> {
    fn map_workflow<U, F>(self, f: F) -> Option<Target<U>>
    where
        F: FnOnce(T) -> Option<U>,
    {
        Some(match self {
            Self::Accept => Target::<U>::Accept,
            Self::Reject => Target::<U>::Reject,
            Self::Workflow(x) => Target::<U>::Workflow(f(x)?),
        })
    }
}

struct Condition {
    rating: u8, // 0..3 for 'x', 'm', 'a' or 's'
    ord: Ordering,
    value: Rating, // comparison value
}

impl Condition {
    fn parse(def: &str) -> Option<Self> {
        let irel = def.find(['<', '>'])?;
        let (srel, rest) = def.split_at(irel);
        let (scond, sval) = rest.split_at(1);

        let rating = rating_index(srel)?;

        let ord = match scond {
            "<" => Ordering::Less,
            ">" => Ordering::Greater,
            _ => return None,
        };

        let value = sval.parse().ok()?;

        Some(Self { rating, ord, value })
    }

    fn match_part(&self, part: &Part) -> bool {
        let v = part.0[self.rating as usize];
        v.cmp(&self.value) == self.ord
    }

    fn split(&self, ps: &PartSpace) -> (PartSpace, PartSpace) {
        match self.ord {
            Ordering::Less => self.split_impl(ps, self.value),
            Ordering::Greater => {
                let (lo, hi) = self.split_impl(ps, self.value + 1);
                (hi, lo)
            }
            _ => panic!("impossible"),
        }
    }

    fn split_impl(&self, ps: &PartSpace, value: Rating) -> (PartSpace, PartSpace) {
        let i = self.rating as usize;
        let r = &ps.0[i];
        if r.1 <= value {
            (PartSpace::empty(), *ps)
        } else if r.0 <= value {
            let mut lo = *ps;
            let mut hi = *ps;
            lo.0[i].1 = value;
            hi.0[i].0 = value;
            (lo, hi)
        } else {
            (*ps, PartSpace::empty())
        }
    }
}

struct Part([Rating; 4]);

impl Part {
    fn parse(s: &str) -> Option<Part> {
        let mut p = Part([0; 4]);
        for frag in s.split(['{', ',', '}']).filter(|p| !p.is_empty()) {
            let (l, r) = frag.split_once('=')?;
            p.0[rating_index(l)? as usize] = r.parse().ok()?;
        }
        Some(p)
    }

    fn sum_rating(&self) -> usize {
        self.0.iter().map(|&i| i as usize).sum()
    }
}

fn rating_index(s: &str) -> Option<u8> {
    Some(match s {
        "x" => 0,
        "m" => 1,
        "a" => 2,
        "s" => 3,
        _ => return None,
    })
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct PartSpace([(Rating, Rating); 4]);

impl PartSpace {
    fn empty() -> Self {
        Self([(0, 0); 4])
    }

    fn range(lo: Rating, hi: Rating) -> Self {
        Self([(lo, hi + 1); 4])
    }

    fn is_empty(&self) -> bool {
        self.0.iter().any(|(lo, hi)| lo >= hi)
    }

    fn total_count(&self) -> usize {
        self.0
            .iter()
            .map(|(lo, hi)| hi.checked_sub(*lo).unwrap_or(0) as usize)
            .product()
    }
}

impl std::fmt::Display for PartSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return write!(f, "{{}}");
        }

        let mut sep = "{ ";
        for (c, r) in std::iter::zip("xmas".chars(), self.0.iter()) {
            write!(f, "{}{}:{}..{}", sep, c, r.0, r.1 - 1)?;
            sep = ", ";
        }
        write!(f, " }}")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let c = Condition::parse("s<1351").unwrap();
        let (l, r) = c.split(&PartSpace::range(1, 4000));
        println!("{l}, {r}");

        let sample = r"
px{a<2006:qkq,m>2090:A,rfg}
pv{a>1716:R,A}
lnx{m>1548:A,A}
rfg{s<537:gd,x>2440:R,A}
qs{s>3448:A,lnx}
qkq{x<1416:A,crn}
crn{x>2662:A,R}
in{s<1351:px,qqz}
qqz{s>2770:qs,m<1801:hdj,R}
gd{a>3333:R,R}
hdj{m>838:A,pv}

{x=787,m=2655,a=1222,s=2876}
{x=1679,m=44,a=2067,s=496}
{x=2036,m=264,a=79,s=2244}
{x=2461,m=1339,a=466,s=291}
{x=2127,m=1623,a=2188,s=1013}
"
        .trim();

        assert_eq!(part1(sample).ok(), Some(19114));
        assert_eq!(part2(sample).ok(), Some(167409079868000));
    }
}
