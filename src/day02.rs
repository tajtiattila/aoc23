use std::str::FromStr;

use anyhow::{anyhow, bail, Result};

pub fn run(input: &str) -> Result<String> {
    let v = input.lines().map(parse_picks).collect::<Result<Vec<_>>>()?;

    Ok(format!("{} {}", p1(&v), p2(&v)))
}

fn p1(games: &[Game]) -> u32 {
    let want = Pick {
        red: 12,
        green: 13,
        blue: 14,
    };
    games
        .iter()
        .filter_map(|g| (g.picks.iter().all(|p| p.all_lt(&want))).then_some(g.no))
        .sum()
}

fn p2(games: &[Game]) -> u32 {
    games
        .iter()
        .map(|g| {
            let mut it = g.picks.iter();
            let x = it.next();
            if x.is_none() {
                return 0;
            }
            let first = *x.unwrap();
            it.fold(first, |acc, pick| acc.accum(pick)).power()
        })
        .sum()
}

#[derive(Debug, Clone)]
struct Game {
    no: u32,
    picks: Vec<Pick>,
}

#[derive(Debug, Clone, Copy)]
struct Pick {
    red: u32,
    green: u32,
    blue: u32,
}

impl Pick {
    pub fn empty() -> Self {
        Self {
            red: 0,
            green: 0,
            blue: 0,
        }
    }

    pub fn all_lt(&self, other: &Self) -> bool {
        self.red <= other.red && self.green <= other.green && self.blue <= other.blue
    }

    pub fn accum(&self, other: &Self) -> Pick {
        Pick {
            red: self.red.max(other.red),
            green: self.green.max(other.green),
            blue: self.blue.max(other.blue),
        }
    }

    pub fn power(&self) -> u32 {
        self.red * self.green * self.blue
    }
}

impl FromStr for Pick {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Pick, Self::Err> {
        let mut p = Pick::empty();

        for e in s.split(',') {
            if let Some((l, r)) = e.trim().split_once(' ') {
                let n = l.parse()?;
                match r {
                    "red" => p.red = n,
                    "green" => p.green = n,
                    "blue" => p.blue = n,
                    _ => bail!("Invalid color in part: {e}"),
                }
            } else {
                bail!("Invalid part: {e}");
            }
        }

        Ok(p)
    }
}

fn parse_picks(line: &str) -> Result<Game> {
    // line format:
    //  Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
    let (gl, gr) = line
        .strip_prefix("Game ")
        .ok_or_else(|| anyhow!("Line is not a game"))?
        .split_once(':')
        .ok_or_else(|| anyhow!("Colon (:) missing in line"))?;

    let game = Game {
        no: gl.parse()?,
        picks: gr
            .split(';')
            .map(|p| p.parse())
            .collect::<Result<Vec<Pick>, _>>()?,
    };

    Ok(game)
}
