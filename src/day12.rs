use anyhow::{anyhow, Context, Result};

pub fn run(input: &str) -> Result<String> {
    /*
    println!("2->4");
    partitions(2, 4, &mut |v| println!("{:?}", v));
    println!("4->2");
    partitions(4, 2, &mut |v| println!("{:?}", v));
    */

    Ok(format!("{}", p1(input)?))
}

fn p1(input: &str) -> Result<usize> {
    input
        .lines()
        //.try_fold(0, |acc, line| Ok(acc + p1_line(line)?))
        .try_fold(0, |acc, line| Ok(acc + Pattern::from(line)?.num_arrang()))
}

struct Pattern {
    pat: Vec<u8>,
    runs: Vec<usize>,
}

impl Pattern {
    fn from(line: &str) -> Result<Self> {
        let (l, r) = line
            .split_once(' ')
            .ok_or_else(|| anyhow!("missing separator in {line}"))?;

        let pat = l.as_bytes().to_vec();

        let runs = r
            .split(',')
            .map(|x| {
                x.parse()
                    .with_context(|| format!("invalid number {x} in {line}"))
            })
            .collect::<Result<Vec<usize>>>()?;

        Ok(Self { pat, runs })
    }

    fn unfold(&self, n: usize) -> Self {
        let mut pat = vec![];
        let mut runs = vec![];
        for i in 0..n {
            if i != 0 {
                pat.push(b'?');
            }
            pat.extend_from_slice(&self.pat);
            runs.extend_from_slice(&self.runs);
        }
        Self { pat, runs }
    }

    fn num_arrang(&self) -> usize {
        // minimum patter length, accounted for one space between runs
        let min_pat_len = self.runs.iter().sum::<usize>() + self.runs.len() - 1;

        // total number of extra spaces needed
        let add_spaces = self.pat.len() - min_pat_len;

        let mut x = PatternCheck {
            pat: &self.pat,
            runs: &self.runs,
            num_possible: 0,
        };

        x.rec(0, 0, add_spaces);

        x.num_possible
    }
}

struct PatternCheck<'a> {
    pat: &'a [u8],
    runs: &'a [usize],

    num_possible: usize,
}

impl PatternCheck<'_> {
    fn rec(&mut self, i_pat: usize, i_run: usize, spaces_left: usize) {
        if i_run == self.runs.len() {
            assert_eq!(i_pat + spaces_left, self.pat.len());
            if Self::is_space(&self.pat[i_pat..]) {
                self.num_possible += 1;
            }
            return;
        }

        for add_space in 0..=spaces_left {
            if let Some(i_pat_next) = self.step(i_pat, add_space, self.runs[i_run]) {
                self.rec(i_pat_next, i_run + 1, spaces_left - add_space);
            }
        }
    }

    fn step(&self, i_pat: usize, add_space: usize, run_len: usize) -> Option<usize> {
        let add_space = add_space + if i_pat == 0 { 0 } else { 1 };
        let i_run = self.step_then(i_pat, add_space, Self::is_space)?;
        self.step_then(i_run, run_len, Self::is_word)
    }

    fn step_then(&self, i: usize, n: usize, mut f: impl FnMut(&[u8]) -> bool) -> Option<usize> {
        f(&self.pat[i..i + n]).then_some(i + n)
    }

    fn is_space(s: &[u8]) -> bool {
        s.iter().all(|&c| c == b'.' || c == b'?')
    }

    fn is_word(s: &[u8]) -> bool {
        s.iter().all(|&c| c == b'#' || c == b'?')
    }
}

fn p1_line(line: &str) -> Result<usize> {
    let (l, r) = line
        .split_once(' ')
        .ok_or_else(|| anyhow!("missing separator in {line}"))?;

    let pat = l.as_bytes();

    let runs = r
        .split(',')
        .map(|x| {
            x.parse()
                .with_context(|| format!("invalid number {x} in {line}"))
        })
        .collect::<Result<Vec<usize>>>()?;

    let min_len = runs.iter().sum::<usize>() + runs.len() - 1;

    let mut work = vec![b'.'; pat.len()];

    let mut acc = 0;

    partitions(pat.len() - min_len, runs.len() + 1, &mut |v| {
        use std::iter::zip;
        let mut i = 0;
        for (&l, &r) in zip(v.iter(), runs.iter()) {
            let l = l + if i == 0 { 0 } else { 1 };
            work[i..i + l].fill(b'.');
            i += l;
            work[i..i + r].fill(b'#');
            i += r;
        }
        work[i..].fill(b'.');

        for (&pc, &wc) in zip(pat.iter(), work.iter()) {
            if pc != b'?' && wc != pc {
                return;
            }
        }

        acc += 1;
    });

    Ok(acc)
}

fn partitions(n: usize, m: usize, f: &mut dyn FnMut(&[usize])) {
    let mut v = vec![0; m];
    parts_impl(&mut v, 0, n, f);
}

fn parts_impl(v: &mut [usize], i: usize, n: usize, f: &mut dyn FnMut(&[usize])) {
    if i + 1 == v.len() {
        v[i] = n;
        f(v)
    } else {
        for x in 0..=n {
            v[i] = x;
            parts_impl(v, i + 1, n - x, f);
        }
    }
}
