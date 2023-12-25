use std::collections::HashMap;

use anyhow::{anyhow, Result};

pub fn run(input: &str) -> Result<String> {
    println!("{}", input.lines().count());
    println!("{:?}", load_input(input)?);
    todo!()
}

fn load_input(input: &str) -> Result<Vec<Vec<usize>>> {
    let mut names = NameMap::new();

    let mut cxn = vec![];
    for line in input.lines() {
        let (l, r) = line
            .split_once(": ")
            .ok_or_else(|| anyhow!("invalid line {line}"))?;

        let l = names.idx(l);
        if cxn.len() <= l {
            cxn.resize(l + 1, vec![]);
        }
        for r in r.split_whitespace().map(|n| names.idx(n)) {
            if cxn.len() <= r {
                cxn.resize(r + 1, vec![]);
            }

            cxn[l].push(r);
            cxn[r].push(l);
        }
    }

    Ok(cxn)
}

struct NameMap<'a>(HashMap<&'a str, usize>);

impl<'a> NameMap<'a> {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn idx(&mut self, s: &'a str) -> usize {
        let next = self.0.len();
        *self.0.entry(s).or_insert(next)
    }
}
