use anyhow::{anyhow, Result};

pub fn run(input: &str) -> Result<String> {
    let (instr, m) = parse_input(input)?;
    Ok(format!("{} {}", p1(instr, &m)?, p2(instr, &m)))
}

fn p1(instr: &str, m: &NodeMap) -> Result<usize> {
    let a = m
        .name_index("AAA")
        .ok_or_else(|| anyhow!("node AAA missing"))?;
    let z = m
        .name_index("ZZZ")
        .ok_or_else(|| anyhow!("node ZZZ missing"))?;

    let mut current = a;
    Ok(instr
        .chars()
        .cycle()
        .filter_map(|c| {
            let p = m.next(current, c);
            current = p.unwrap_or(current);
            p
        })
        .take_while(|&p| p != z)
        .count()
        + 1)
}

fn p2(instr: &str, m: &NodeMap) -> usize {
    /*
    for p in 0..m.node_count() {
        if m.is_start_2(p) {
            println!("{p} {}", m.cycle_len(p, instr));
            let v = m.goal_steps(p, instr).take(20).collect::<Vec<_>>();
            for (n, d) in v.iter().scan(0, |state, &n| {
                let d = n - *state;
                *state = n;
                Some((n, d))
            }) {
                println!("  {n} {d}");
            }
        }
    }
    */

    (0..m.node_count())
        .filter_map(|p| m.is_start_2(p).then(|| m.cycle_len(p, instr)))
        .fold(1, num::integer::lcm)
}

fn parse_input(input: &str) -> Result<(&str, NodeMap)> {
    let mut it = input.lines();
    let instr = it.next().ok_or_else(|| anyhow!("instruction missing"))?;

    it.next().ok_or_else(|| anyhow!("map missing"))?;

    let mut v = it.map(Node::from).collect::<Result<Vec<_>>>()?;

    Ok((instr, NodeMap::from(&mut v)?))
}

struct NodeMap<'a>(Vec<NodeMapEntry<'a>>);

struct NodeMapEntry<'a> {
    name: &'a str,
    left: usize,
    right: usize,
}

impl NodeMap<'_> {
    fn from<'a>(src: &mut [Node<'a>]) -> Result<NodeMap<'a>> {
        src.sort_by_key(|n| n.name);

        let v = src
            .iter()
            .map(|n| {
                let name = n.name;
                let left = src
                    .binary_search_by_key(&n.left, |m| m.name)
                    .map_err(|_| anyhow!("node {name}: left node {} is invalid", n.left))?;
                let right = src
                    .binary_search_by_key(&n.right, |m| m.name)
                    .map_err(|_| anyhow!("node {name}: right node {} is invalid", n.right))?;
                Ok(NodeMapEntry { name, left, right })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(NodeMap(v))
    }

    fn node_count(&self) -> usize {
        self.0.len()
    }

    fn name_index(&self, name: &str) -> Option<usize> {
        self.0.binary_search_by_key(&name, |n| n.name).ok()
    }

    fn is_start_2(&self, p: usize) -> bool {
        self.has_suffix(p, 'A')
    }

    fn is_end_2(&self, p: usize) -> bool {
        self.has_suffix(p, 'Z')
    }

    fn has_suffix(&self, p: usize, suffix: char) -> bool {
        self.0
            .get(p)
            .map(|m| m.name.ends_with(suffix))
            .unwrap_or(false)
    }

    fn next(&self, p: usize, c: char) -> Option<usize> {
        if p < self.0.len() {
            match c {
                'L' => Some(self.0[p].left),
                'R' => Some(self.0[p].right),
                _ => None,
            }
        } else {
            None
        }
    }

    fn cycle_len(&self, p: usize, instr: &str) -> usize {
        // NOTE(ata): thankfully all cycles are 'full cycles'
        // i.e. they get back to the starting point,
        // which simplifies lcm calculation.
        let mut it = self.goal_steps(p, instr);
        let c0 = it.next();
        let c1 = it.next();
        if let (Some(c0), Some(c1)) = (c0, c1) {
            let (a, b) = (c0 + 1, c1 - c0);
            if a == b {
                return a;
            }
        }

        panic!("invaild cycle");
    }

    fn goal_steps<'a>(&'a self, p: usize, instr: &'a str) -> impl Iterator<Item = usize> + 'a {
        let mut cur = Some(p);
        instr
            .chars()
            .cycle()
            .enumerate()
            .map(move |(i, c)| {
                if let Some(p) = cur {
                    cur = self.next(p, c);
                }
                cur.map(|p| (i, p))
            })
            .take_while(|x| x.is_some())
            .map(|x| x.unwrap())
            .filter_map(|(i, p)| self.is_end_2(p).then_some(i))
    }
}

#[derive(Debug, Copy, Clone)]
struct Node<'a> {
    name: &'a str,
    left: &'a str,
    right: &'a str,
}

impl Node<'_> {
    fn from(line: &str) -> Result<Node> {
        Self::from_impl(line).ok_or_else(|| anyhow!("invalid node: {line}"))
    }

    fn from_impl(line: &str) -> Option<Node> {
        let mut it = line
            .split([' ', '=', '(', ')', ','])
            .filter(|e| !e.is_empty());
        let name = it.next()?;
        let left = it.next()?;
        let right = it.next()?;
        (it.next().is_none()).then_some(Node { name, left, right })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn p1_works() {
        let sample = "\
RL

AAA = (BBB, CCC)
BBB = (DDD, EEE)
CCC = (ZZZ, GGG)
DDD = (DDD, DDD)
EEE = (EEE, EEE)
GGG = (GGG, GGG)
ZZZ = (ZZZ, ZZZ)
";
        let p1r = |src| {
            let (i, m) = parse_input(src).ok()?;
            p1(i, &m).ok()
        };
        assert_eq!(p1r(sample), Some(2));

        let s2 = "\
LLR

AAA = (BBB, BBB)
BBB = (AAA, ZZZ)
ZZZ = (ZZZ, ZZZ)
";
        assert_eq!(p1r(s2), Some(6));
    }

    #[test]
    fn p2_works() {
        let sample = "\
LR

11A = (11B, XXX)
11B = (XXX, 11Z)
11Z = (11B, XXX)
22A = (22B, XXX)
22B = (22C, 22C)
22C = (22Z, 22Z)
22Z = (22B, 22B)
XXX = (XXX, XXX)
";
        let (i, m) = parse_input(sample).unwrap();
        assert_eq!(p2(i, &m), 6);
    }
}
