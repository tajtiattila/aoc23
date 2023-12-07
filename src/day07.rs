use anyhow::{anyhow, Result};
use std::cmp::Reverse;

pub fn run(input: &str) -> Result<String> {
    Ok(format!("{} {}", p1(input)?, p2(input)?))
}

fn p1(input: &str) -> Result<i64> {
    let mut v = games(input)?;
    v.sort_by_key(|x| x.hand);
    Ok(v.iter()
        .enumerate()
        .map(|(i, g)| (i + 1) as i64 * g.bid as i64)
        .sum())
}

fn p2(input: &str) -> Result<i64> {
    Ok(0)
}

struct Game {
    hand: Hand,
    bid: i32,
}

fn games(input: &str) -> Result<Vec<Game>> {
    input
        .lines()
        .map(|line| {
            let (l, r) = line
                .split_once(' ')
                .ok_or_else(|| anyhow!("invalid line: {line}"))?;
            let hand = l.parse()?;
            let bid = r.parse()?;
            Ok(Game { hand, bid })
        })
        .collect()
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Hand {
    type_: HandType,
    cardvs: [u8; 5],
}

impl std::str::FromStr for Hand {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let v = s.chars().map(to_card_idx).collect::<Result<Vec<_>, _>>()?;

        let cardvs: [u8; 5] = v.try_into().map_err(|_| anyhow!("invalid hand length"))?;
        let type_ = HandType::from(&cardvs);

        Ok(Self { cardvs, type_ })
    }
}

const CARDS: &[char] = &[
    '2', '3', '4', '5', '6', '7', '8', '9', 'T', 'J', 'Q', 'K', 'A',
];

fn to_card_idx(c: char) -> Result<u8> {
    CARDS
        .iter()
        .position(|&x| x == c)
        .map(|x| x as u8)
        .ok_or_else(|| anyhow!("invalid card {c}"))
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

impl HandType {
    fn from(hand: &[u8]) -> HandType {
        if hand.len() != 5 {
            panic!("invalid hand input");
        }

        let mut v = vec![(0, 0); CARDS.len()];
        v.iter_mut().enumerate().for_each(|(i, x)| x.0 = i);
        for x in hand {
            v[*x as usize].1 += 1;
        }

        v.retain(|&(_, n)| n > 0);
        v.sort_by_key(|&(_, n)| Reverse(n));

        let mut it = v.iter().map(|&(_, n)| n);

        use HandType::*;
        match it.next() {
            Some(5) => FiveOfAKind,
            Some(4) => FourOfAKind,
            Some(3) => {
                if it.next().unwrap_or(0) > 1 {
                    FullHouse
                } else {
                    ThreeOfAKind
                }
            }
            Some(2) => {
                if it.next().unwrap_or(0) > 1 {
                    TwoPair
                } else {
                    OnePair
                }
            }
            _ => HighCard,
        }
    }
}
