use std::collections::{HashSet, VecDeque};

use anyhow::{anyhow, Result};

pub fn run(input: &str) -> Result<String> {
    Ok(format!("{} {}", p1(input)?, p2(input)?))
}

fn p1(input: &str) -> Result<usize> {
    input.lines().try_fold(0, |total, line| {
        Card::parse(line).map(|card| total + card.score())
    })
}

fn p2(input: &str) -> Result<usize> {
    let mut nexts = VecDeque::new();

    input.lines().try_fold(0, |total, line| {
        Card::parse(line).map(|card| {
            let nc = 1 + nexts.pop_front().unwrap_or(0);

            if card.wins > nexts.len() {
                nexts.resize(card.wins, 0);
            }
            nexts.iter_mut().take(card.wins).for_each(|n| *n += nc);

            total + nc
        })
    })
}

#[allow(unused)]
struct Card {
    num: u32,
    wins: usize,
}

impl Card {
    fn parse(line: &str) -> Result<Self> {
        let (ns, lrs) = line
            .strip_prefix("Card")
            .ok_or_else(|| anyhow!("invalid card"))?
            .split_once(':')
            .ok_or_else(|| anyhow!("missing colon (':')"))?;
        let (ls, rs) = lrs
            .split_once('|')
            .ok_or_else(|| anyhow!("missing pipe ('|')"))?;
        let num = ns.trim().parse()?;
        let set = ls
            .split_whitespace()
            .map(|v| v.parse())
            .collect::<Result<HashSet<u32>, _>>()?;
        let wins = rs.split_whitespace().try_fold(0, |total, s| {
            s.parse().map(|n| total + set.contains(&n) as usize)
        })?;
        Ok(Card { num, wins })
    }

    fn score(&self) -> usize {
        if self.wins > 0 {
            1 << (self.wins - 1)
        } else {
            0
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cards() {
        let src = "\
Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11
";
        assert_eq!(p1(src).ok(), Some(13));
        assert_eq!(p2(src).ok(), Some(30));
    }
}
