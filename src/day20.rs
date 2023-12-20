use anyhow::{anyhow, Result};
use std::collections::{HashMap, VecDeque};

pub fn run(input: &str) -> Result<String> {
    let dbg = crate::Cli::global().verbose;
    if dbg {
        Circuit::load(input)?.print_dot_graph();
    }

    let p1 = part1(input)?;
    let p2 = part2(input)?;
    Ok(format!("{p1} {p2}"))
}

fn part1(input: &str) -> Result<usize> {
    Circuit::load(input).map(|c| c.button_pulses_hilo(1000))
}

fn part2(input: &str) -> Result<usize> {
    Circuit::load(input).map(|c| c.button_presses_needed())
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
        let mut links = vec![];

        let mut add_name = |n: &str| {
            if names.get(n).is_none() {
                names.insert(n.to_string(), names.len() as Name);
                links.push(Link {
                    name: n.to_string(),
                    type_: LinkType::Special,
                    offset: 0,
                    sources: vec![],
                    targets: vec![],
                })
            }
        };

        src_links.iter().for_each(|sl| add_name(sl.name));
        src_links
            .iter()
            .flat_map(|sl| sl.targets.iter())
            .for_each(|n| add_name(n));

        let mut vsrc = vec![Vec::new(); names.len()];
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
                    LinkType::Special => 0,
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

    fn button_pulses_hilo(&self, npress: usize) -> usize {
        let brc = *self.names.get("broadcaster").unwrap_or(&0);

        let mut s = self.new_state();
        let (sumlo, sumhi) = (0..npress).fold((0, 0), |(sumlo, sumhi), _| {
            let (lo, hi, _) = self.run(&mut s, brc, None);
            (sumlo + lo, sumhi + hi)
        });
        sumlo * sumhi
    }

    fn button_presses_needed(&self) -> usize {
        let brc = *self.names.get("broadcaster").unwrap_or(&0);
        let rx = self.names.get("rx").copied();
        if rx.is_none() {
            return 0;
        }

        let dbg = cfg!(test) || crate::Cli::global().verbose;
        if dbg {
            for l in self.flipflops_labels() {
                println!("{:8} {}", "", l);
            }
        }

        let mut s = self.new_state();
        let mut npress = 0;
        let mut pow2 = 1;
        loop {
            npress += 1;
            let (_, _, rx_low_seen) = self.run(&mut s, brc, rx);
            if rx_low_seen {
                return npress;
            }

            if dbg && npress == pow2 {
                pow2 *= 2;
                println!("{:8} {}", npress, self.flipflops_string(&s));
            }
        }
    }

    fn new_state(&self) -> Vec<bool> {
        vec![false; self.state_len]
    }

    // run circuit, returning number of low/high pulses
    fn run(&self, state: &mut [bool], brc: u16, rx: Option<u16>) -> (usize, usize, bool) {
        let mut pulses = VecDeque::from([(brc, brc, false)]);
        let mut n_pulses = [0, 0];
        let mut rx_low_seen = false;
        while let Some((src, cur, pulse)) = pulses.pop_front() {
            n_pulses[pulse as usize] += 1;

            if !pulse && Some(cur) == rx {
                rx_low_seen = true;
            }

            let link = &self.links[cur as usize];
            let out_pulse = match link.type_ {
                LinkType::Special => Some(pulse),
                LinkType::FlipFlop => {
                    // When low pulse received: toggle state and output it,
                    // otherwise do nothing.
                    if !pulse {
                        let s = &mut state[link.offset];
                        *s = !*s;
                        Some(*s)
                    } else {
                        None
                    }
                }
                LinkType::Conjunction => {
                    // Update source state, when all sources were high output low,
                    // otherwise output high.
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

        (n_pulses[0], n_pulses[1], rx_low_seen)
    }

    fn flipflops_labels(&self) -> Vec<String> {
        let names = || {
            self.links
                .iter()
                .filter_map(|link| (link.type_ == LinkType::FlipFlop).then_some(&link.name as &str))
        };
        let n_lines = names().map(|n| n.chars().count()).max().unwrap_or(0);
        (0..n_lines)
            .map(|i| names().map(|n| n.chars().nth(i).unwrap_or(' ')).collect())
            .collect()
    }

    fn flipflops_string(&self, state: &[bool]) -> String {
        let mut ss = String::new();
        for link in &self.links {
            if link.type_ == LinkType::FlipFlop {
                let s = &state[link.offset];
                ss.push(if *s { '#' } else { '.' });
            }
        }
        ss
    }

    fn print_dot_graph(&self) {
        println!("digraph G {{");
        for l in &self.links {
            match l.type_ {
                LinkType::FlipFlop => println!("  {} [shape=box];", l.name),
                LinkType::Special => println!("  {} [shape=diamond];", l.name),
                _ => {}
            }
            if !l.targets.is_empty() {
                print!("  {} -> ", l.name);
                let mut sep = "{";
                for &t in &l.targets {
                    print!("{} {}", sep, self.links[t as usize].name);
                    sep = ",";
                }
                println!(" }};");
            }
        }
        println!("}}\n");
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

#[derive(Debug, Clone, PartialEq)]
enum LinkType {
    Special, // broadcaster and rx
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
            _ => (LinkType::Special, l),
        };

        let targets = r.split(", ").collect();

        Some(SourceLink {
            type_,
            name,
            targets,
        })
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
