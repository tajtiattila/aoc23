use anyhow::{anyhow, Result};
use std::cmp::Reverse;

pub fn run(input: &str) -> Result<String> {
    let p1 = play(GameType::Simple, input)?;
    let p2 = play(GameType::WithJoker, input)?;
    Ok(format!("{p1} {p2}"))
}

fn play(t: GameType, input: &str) -> Result<i64> {
    let mut v = games(t, input)?;
    v.sort_by_key(|x| x.hand);
    Ok(v.iter()
        .enumerate()
        .map(|(i, g)| (i + 1) as i64 * g.bid as i64)
        .sum())
}

struct Game {
    hand: Hand,
    bid: i32,
}

fn games(t: GameType, input: &str) -> Result<Vec<Game>> {
    input
        .lines()
        .map(|line| {
            let (l, r) = line
                .split_once(' ')
                .ok_or_else(|| anyhow!("invalid line: {line}"))?;
            let hand = Hand::from(t, l)?;
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

impl Hand {
    fn from(t: GameType, s: &str) -> Result<Self> {
        let v = s
            .chars()
            .map(|c| t.to_card_index(c))
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| anyhow!("invalid card in {s}"))?;

        let cardvs: [u8; 5] = v.try_into().map_err(|_| anyhow!("invalid hand length"))?;
        let type_ = t.hand_type(&cardvs);

        Ok(Self { cardvs, type_ })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum GameType {
    Simple,
    WithJoker,
}

impl GameType {
    fn cards(self) -> &'static str {
        match self {
            Self::Simple => "23456789TJQKA",
            Self::WithJoker => "J23456789TQKA",
        }
    }

    fn to_card_index(self, c: char) -> Option<u8> {
        self.cards().chars().position(|x| x == c).map(|x| x as u8)
    }

    fn joker_index(self) -> Option<u8> {
        match self {
            Self::Simple => None,
            Self::WithJoker => Some(0),
        }
    }

    fn hand_type(self, hand: &[u8]) -> HandType {
        let mut v = vec![(0, 0); self.cards().len()];
        v.iter_mut().enumerate().for_each(|(i, x)| x.0 = i);

        let mut njoker = 0;

        for &x in hand {
            if Some(x) == self.joker_index() {
                njoker += 1;
            } else {
                v[x as usize].1 += 1;
            }
        }

        // Handle five or more jokers.
        if njoker >= 5 {
            return HandType::FiveOfAKind;
        }

        v.retain(|&(_, n)| n > 0);
        v.sort_by_key(|&(_, n)| Reverse(n));
        v[0].1 += njoker;

        let mut it = v.iter().map(|&(_, n)| n);

        use HandType::*;
        match it.next() {
            Some(x) if x >= 5 => FiveOfAKind,
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
