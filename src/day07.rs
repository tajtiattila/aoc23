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
        let cards = t.cards();

        let to_card_idx = |c| {
            cards
                .iter()
                .position(|&x| x == c)
                .map(|x| x as u8)
                .ok_or_else(|| anyhow!("invalid card {c}"))
        };

        let v = s.chars().map(to_card_idx).collect::<Result<Vec<_>, _>>()?;

        let cardvs: [u8; 5] = v.try_into().map_err(|_| anyhow!("invalid hand length"))?;
        let type_ = HandType::from(t, &cardvs);

        Ok(Self { cardvs, type_ })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum GameType {
    Simple,
    WithJoker,
}

impl GameType {
    fn cards(self) -> &'static [char] {
        const CARDS_SIMPL: &[char] = &[
            '2', '3', '4', '5', '6', '7', '8', '9', 'T', 'J', 'Q', 'K', 'A',
        ];
        const CARDS_JOKER: &[char] = &[
            'J', '2', '3', '4', '5', '6', '7', '8', '9', 'T', 'Q', 'K', 'A',
        ];
        match self {
            Self::Simple => CARDS_SIMPL,
            Self::WithJoker => CARDS_JOKER,
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

impl HandType {
    pub fn from(t: GameType, hand: &[u8]) -> HandType {
        if hand.len() != 5 {
            panic!("invalid hand input");
        }

        match t {
            GameType::Simple => Self::from_simple(hand),
            GameType::WithJoker => Self::from_joker(hand),
        }
    }

    fn from_simple(hand: &[u8]) -> HandType {
        if hand.len() != 5 {
            panic!("invalid hand input");
        }

        let mut v = vec![(0, 0); (GameType::Simple).cards().len()];
        v.iter_mut().enumerate().for_each(|(i, x)| x.0 = i);
        for x in hand {
            v[*x as usize].1 += 1;
        }

        v.retain(|&(_, n)| n > 0);
        v.sort_by_key(|&(_, n)| Reverse(n));

        Self::deduce(&v)
    }

    fn from_joker(hand: &[u8]) -> HandType {
        if hand.len() != 5 {
            panic!("invalid hand input");
        }

        let mut v = vec![(0, 0); (GameType::WithJoker).cards().len()];
        let mut njoker = 0;
        v.iter_mut().enumerate().for_each(|(i, x)| x.0 = i);
        for &x in hand {
            if x == 0 {
                njoker += 1;
            } else {
                v[x as usize].1 += 1;
            }
        }

        if njoker == 5 {
            return HandType::FiveOfAKind;
        }

        v.retain(|&(_, n)| n > 0);
        v.sort_by_key(|&(_, n)| Reverse(n));
        v[0].1 += njoker;

        Self::deduce(&v)
    }

    fn deduce(v: &[(usize, i32)]) -> HandType {
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
