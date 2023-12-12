use anyhow::{anyhow, Context, Result};

pub fn run(input: &str) -> Result<String> {
    let p1 = proc(input, 1)?;
    let p2 = proc(input, 5)?;

    Ok(format!("{p1} {p2}"))
}

fn proc(input: &str, n_copies: usize) -> Result<usize> {
    let pats = input
        .lines()
        .map(Pattern::from)
        .collect::<Result<Vec<_>>>()?;

    let dbg = cfg!(test) || crate::Cli::global().verbose;
    if dbg {
        println!("\nusing {n_copies} copies");
    }

    let now = std::time::Instant::now();

    Ok(pats.iter().enumerate().fold(0, |acc, (i, pat)| {
        if dbg {
            println!(
                "{} {}/{} {} {:?}",
                super::fmt_duration(now.elapsed()),
                i,
                pats.len(),
                String::from_utf8_lossy(&pat.pat),
                pat.runs
            );
        }
        acc + pat.num_arrg_unfolded(n_copies)
    }))
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

    fn num_arrg_unfolded(&self, n_copies: usize) -> usize {
        match n_copies {
            0 => 0,
            1 => self.num_arrg(),
            _ => self.unfold(n_copies).num_arrg(),
        }
    }

    fn num_arrg(&self) -> usize {
        let mut x = PatternCheck {
            pat: &self.pat,
            runs: &self.runs,
        };
        x.run()
    }

    fn unfold(&self, n_copies: usize) -> Self {
        let mut pat = vec![];
        let mut runs = vec![];
        for i in 0..n_copies {
            if i != 0 {
                pat.push(b'?');
            }
            pat.extend_from_slice(&self.pat);
            runs.extend_from_slice(&self.runs);
        }
        Self { pat, runs }
    }
}

struct PatternCheck<'a> {
    pat: &'a [u8],
    runs: &'a [usize],
}

impl PatternCheck<'_> {
    fn rec(&mut self, i_pat: usize, i_run: usize, spaces_left: usize) -> usize {
        if i_run == self.runs.len() {
            assert_eq!(i_pat + spaces_left, self.pat.len());
            if Self::is_space(&self.pat[i_pat..]) {
                return 1;
            }
        }

        let mut total = 0;

        for add_space in 0..=spaces_left {
            if let Some(i_pat_next) = self.step(i_pat, add_space, self.runs[i_run]) {
                total += self.rec(i_pat_next, i_run + 1, spaces_left - add_space);
            }
        }

        total
    }

    fn run(&mut self) -> usize {
        // minimum patter length, accounted for one space between runs
        let min_pat_len = self.runs.iter().sum::<usize>() + self.runs.len() - 1;

        // total number of extra spaces needed
        let spaces_total = self.pat.len() - min_pat_len;

        self.rec(0, 0, spaces_total)
        /*
        let dbg = cfg!(test) || crate::Cli::global().verbose;
        if dbg {
            println!(
                "add {spaces_total} spaces in {} buckets",
                self.runs.len() + 1
            );
        }

        let mut stack = vec![(0, 0, spaces_total)];

        let mut num_possible = 0;
        while let Some((i_pat, i_run, spaces_left)) = stack.pop() {
            if i_run == self.runs.len() {
                assert_eq!(i_pat + spaces_left, self.pat.len());
                if Self::is_space(&self.pat[i_pat..]) {
                    num_possible += 1;
                }
            } else {
                for add_space in 0..=spaces_left {
                    if let Some(i_pat_next) = self.step(i_pat, add_space, self.runs[i_run]) {
                        stack.push((i_pat_next, i_run + 1, spaces_left - add_space));
                    }
                }
            }
        }

        num_possible
        */
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let test = |str| -> usize { Pattern::from(str).unwrap().num_arrg_unfolded(5) };

        assert_eq!(test("???.### 1,1,3"), 1);
        assert_eq!(test(".??..??...?##. 1,1,3"), 16384);
        assert_eq!(test("?#?#?#?#?#?#?#? 1,3,1,6"), 1);
        assert_eq!(test("????.#...#... 4,1,1"), 16);
        assert_eq!(test("????.######..#####. 1,6,5"), 2500);
        assert_eq!(test("?###???????? 3,2,1"), 506250);
    }
}
