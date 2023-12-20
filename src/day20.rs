use anyhow::{anyhow, bail, Result};
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
    Circuit::load(input).and_then(|c| c.button_presses_needed())
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

        let mut state = self.new_state();
        let mut conj_hi = self.new_state();
        let (sumlo, sumhi) = (0..npress).fold((0, 0), |(sumlo, sumhi), _| {
            let (lo, hi) = self.run(&mut state, &mut conj_hi, brc);
            (sumlo + lo, sumhi + hi)
        });
        sumlo * sumhi
    }

    fn button_presses_needed(&self) -> Result<usize> {
        let brc = *self.names.get("broadcaster").unwrap_or(&0);
        let rx = self
            .names
            .get("rx")
            .copied()
            .ok_or_else(|| anyhow!("rx not found"))?;

        let n = self.links[rx as usize].sources.len();
        if n != 1 {
            bail!("rx has {n} sources");
        }

        let ilast = self.links[rx as usize].sources[0];
        let last = &self.links[ilast as usize];
        if last.sources.len() < 2 {
            bail!(
                "last node {} in front of rx has {} sources",
                last.name,
                last.sources.len()
            );
        }

        let mut final_inputs = last
            .sources
            .iter()
            .map(|&si| {
                let sl = &self.links[si as usize];
                PulseInfo {
                    name: sl.name.clone(),
                    offset: sl.offset,
                    first_high: None,
                }
            })
            .collect::<Vec<_>>();

        let dbg = cfg!(test) || crate::Cli::global().verbose;
        if dbg {
            let names = final_inputs
                .iter()
                .map(|i| i.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            println!("looking for inputs {names}");
        }

        let mut state = self.new_state();
        let mut conj_hi = self.new_state();
        let mut npress = 0;
        loop {
            npress += 1;
            conj_hi.fill(false);
            self.run(&mut state, &mut conj_hi, brc);

            let done = final_inputs.iter_mut().fold(true, |acc, inf| {
                if inf.first_high.is_none() && conj_hi[inf.offset] {
                    inf.first_high = Some(npress);
                    if dbg {
                        println!("  {}: {}", inf.name, npress);
                    }
                }
                acc && inf.first_high.is_some()
            });

            if done {
                break;
            }
        }

        Ok(final_inputs
            .iter()
            .map(|inf| inf.first_high.unwrap())
            .product())
    }

    fn new_state(&self) -> Vec<bool> {
        vec![false; self.state_len]
    }

    // run circuit, returning number of low/high pulses
    fn run(&self, state: &mut [bool], conj_high: &mut [bool], brc: u16) -> (usize, usize) {
        let mut pulses = VecDeque::from([(brc, brc, false)]);
        let mut n_pulses = [0, 0];
        while let Some((src, cur, pulse)) = pulses.pop_front() {
            n_pulses[pulse as usize] += 1;

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
                    // First update remembered source state.
                    // Then output low when all sources were high,
                    // otherwise output high.
                    let s = &mut state[link.offset..][..link.sources.len()];
                    let (_, b) = std::iter::zip(link.sources.iter(), s.iter_mut())
                        .find(|(&j, _)| j == src)
                        .unwrap();
                    *b = pulse;
                    let pulse_high = !s.iter().all(|x| *x);
                    if pulse_high {
                        conj_high[link.offset] = true;
                    }
                    Some(pulse_high)
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

#[derive(Debug, Clone)]
struct PulseInfo {
    name: String,
    offset: usize,             // offset (conjunction result) to check
    first_high: Option<usize>, // first high seen
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
