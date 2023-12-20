use anyhow::{anyhow, Result};
use std::collections::{HashMap, VecDeque};

pub fn run(input: &str) -> Result<String> {
    Ok(format!("{}", part1(input)?))
}

fn part1(input: &str) -> Result<usize> {
    Circuit::load(input).map(|c| c.button_pulses_result(1000))
}

type Name = u16;

#[derive(Debug, Clone)]
struct Circuit {
    names: HashMap<String, Name>,
    links: Vec<Link>,

    state_len: usize,
}

impl Circuit {
    fn load(input: &str) -> Result<Circuit> {
        let src_links = input
            .lines()
            .map(|line| SourceLink::parse(line).ok_or_else(|| anyhow!("error parsing {line}")))
            .collect::<Result<Vec<_>>>()?;

        let mut names = HashMap::new();
        let mut add_name = |n: &str| {
            if names.get(n).is_none() {
                names.insert(n.to_string(), names.len() as Name);
            }
        };

        src_links.iter().for_each(|sl| add_name(sl.name));
        src_links
            .iter()
            .flat_map(|sl| sl.targets.iter())
            .for_each(|n| add_name(n));

        let mut vsrc = vec![Vec::new(); names.len()];
        let mut links = vec![
            Link {
                name: String::new(),
                type_: LinkType::Broadcaster,
                offset: 0,
                sources: vec![],
                targets: vec![]
            };
            names.len()
        ];
        for sl in src_links {
            let i = names.get(sl.name).copied().unwrap();

            let targets = sl
                .targets
                .into_iter()
                .map(|t| {
                    let t = *names.get(t).unwrap();
                    vsrc[t as usize].push(i);
                    t
                })
                .collect();

            let l = &mut links[i as usize];
            l.name = sl.name.to_string();
            l.type_ = sl.type_;
            l.targets = targets;
        }

        let state_len =
            std::iter::zip(links.iter_mut(), vsrc.into_iter()).fold(0, |acc, (link, sources)| {
                let state_len = match link.type_ {
                    LinkType::Broadcaster => 0,
                    LinkType::FlipFlop => 1,
                    LinkType::Conjunction => sources.len(),
                };

                link.offset = acc;
                link.sources = sources;

                acc + state_len
            });

        Ok(Circuit {
            names,
            links,
            state_len,
        })
    }

    fn button_pulses_result(&self, npress: usize) -> usize {
        self.show_info();
        let mut s = self.new_state();
        let (sumlo, sumhi) = (0..npress).fold((0, 0), |(sumlo, sumhi), _| {
            let (lo, hi) = self.run(&mut s);
            (sumlo + lo, sumhi + hi)
        });
        sumlo * sumhi
    }

    fn new_state(&self) -> Vec<bool> {
        vec![false; self.state_len]
    }

    // run circuit, returning number of low/high pulses
    fn run(&self, state: &mut [bool]) -> (usize, usize) {
        let brc = *self.names.get("broadcaster").unwrap_or(&0);
        let mut pulses = VecDeque::from([(brc, brc, false)]);
        let mut n_pulses = [0, 0];
        while let Some((src, cur, pulse)) = pulses.pop_front() {
            n_pulses[pulse as usize] += 1;

            let link = &self.links[cur as usize];
            let out_pulse = match link.type_ {
                LinkType::Broadcaster => Some(pulse),
                LinkType::FlipFlop => {
                    if !pulse {
                        // Low pulse received: toggle state and output it
                        let s = &mut state[link.offset];
                        *s = !*s;
                        Some(*s)
                    } else {
                        None
                    }
                }
                LinkType::Conjunction => {
                    let s = &mut state[link.offset..][..link.sources.len()];
                    let (_, b) = std::iter::zip(link.sources.iter(), s.iter_mut())
                        .find(|(&j, _)| j == src)
                        .unwrap();
                    *b = pulse;
                    Some(!s.iter().all(|x| *x))
                }
            };

            if let Some(pulse) = out_pulse {
                for &t in &link.targets {
                    pulses.push_back((cur, t, pulse));
                }
            }
        }
        (n_pulses[0], n_pulses[1])
    }

    fn show_info(&self) {
        for l in &self.links {
            let mut line = format!("{}: (", &l.name);
            for (i, &n) in l.sources.iter().enumerate() {
                if i != 0 {
                    line.push_str(", ");
                }
                line.push_str(&self.links[n as usize].name);
            }
            line.push(')');
            for (i, &n) in l.targets.iter().enumerate() {
                if i == 0 {
                    line.push_str(" -> ");
                } else {
                    line.push_str(", ");
                }
                line.push_str(&self.links[n as usize].name);
            }
            println!("{line}");
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
struct Link {
    name: String,
    type_: LinkType,
    offset: usize,
    sources: Vec<Name>,
    targets: Vec<Name>,
}

#[derive(Debug, Clone)]
enum LinkType {
    Broadcaster,
    FlipFlop,
    Conjunction,
}

struct SourceLink<'a> {
    type_: LinkType,
    name: &'a str,
    targets: Vec<&'a str>,
}

impl<'a> SourceLink<'a> {
    fn parse(line: &str) -> Option<SourceLink> {
        let (l, r) = line.split_once(" -> ")?;

        let (type_, name) = match &l[0..1] {
            "%" => (LinkType::FlipFlop, &l[1..]),
            "&" => (LinkType::Conjunction, &l[1..]),
            _ => (LinkType::Broadcaster, l),
        };

        let targets = r.split(", ").collect();

        Some(SourceLink {
            type_,
            name,
            targets,
        })
    }

    fn names(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.name).chain(self.targets.iter().copied())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let sample1 = "\
broadcaster -> a, b, c
%a -> b
%b -> c
%c -> inv
&inv -> a
";
        let sample2 = "\
broadcaster -> a
%a -> inv, con
&inv -> b
%b -> con
&con -> output
";

        assert_eq!(part1(sample1).ok(), Some(32000000));
        assert_eq!(part1(sample2).ok(), Some(11687500));
    }
}
