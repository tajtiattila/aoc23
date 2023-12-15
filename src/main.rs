use anyhow::Result;
use clap::Parser;
use once_cell::sync::OnceCell;
use std::collections::HashSet;
use std::time::{Duration, Instant};

const AOC_YEAR: u32 = 23;

mod day01;
mod day02;
mod day03;
mod day04;
mod day05;
mod day06;
mod day07;
mod day08;
mod day09;
mod day10;
mod day11;
mod day12;
mod day13;
mod day14;
mod day15;

fn day_funcs() -> Vec<DayFunc> {
    vec![
        (day01::run as DayFunc),
        (day02::run as DayFunc),
        (day03::run as DayFunc),
        (day04::run as DayFunc),
        (day05::run as DayFunc),
        (day06::run as DayFunc),
        (day07::run as DayFunc),
        (day08::run as DayFunc),
        (day09::run as DayFunc),
        (day10::run as DayFunc),
        (day11::run as DayFunc),
        (day12::run as DayFunc),
        (day13::run as DayFunc),
        (day14::run as DayFunc),
        (day15::run as DayFunc),
    ]
}

mod util;
use util::InputSource;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    all: bool,

    days: Vec<usize>,
}

impl Cli {
    #[allow(unused)]
    pub fn global() -> &'static Cli {
        CLI_INSTANCE.get().expect("CLI is not initialized")
    }
}

static CLI_INSTANCE: OnceCell<Cli> = OnceCell::new();

fn main() -> anyhow::Result<()> {
    let is = InputSource::new()?;

    let cli = Cli::parse();

    let dfs = get_day_funcs(&cli);

    CLI_INSTANCE.set(cli).unwrap();

    for (i, f) in dfs {
        let r = is.get(i);
        let now = Instant::now();
        let r = r.and_then(|s| f(&s));
        print!("Day {:2}: ", i);
        match r {
            Ok(result) => println!("{}  ({})", result, fmt_duration(now.elapsed())),
            Err(e) => {
                println!();
                eprintln!("{}", e);
            }
        }
    }

    Ok(())
}

fn fmt_duration(d: Duration) -> String {
    let ms = d.as_secs_f64() * 1000.0;
    if ms < 100.0 {
        return format!("{:.1}ms", ms);
    }
    let nsec = d.as_secs();
    let h = nsec / 3600;
    let m = (nsec / 60) % 60;
    let s = nsec % 60;
    let ms = d.as_millis() % 1000;
    let mut fmt = String::new();
    if h > 0 {
        fmt.push_str(&format!("{}h", h))
    }
    if h > 0 || m > 0 {
        fmt.push_str(&format!("{}m", m))
    }
    fmt.push_str(&format!("{}.{:03}s", s, ms));
    fmt
}

pub fn verbose() -> bool {
    CLI_INSTANCE.get().map(|cli| cli.verbose).unwrap_or(true)
}

type DayFunc = fn(&str) -> Result<String>;

fn get_day_funcs(cli: &Cli) -> Vec<(usize, DayFunc)> {
    let v: Vec<(usize, DayFunc)> = day_funcs()
        .into_iter()
        .enumerate()
        .map(|(n, f)| (n + 1, f))
        .collect();
    if !cli.days.is_empty() {
        let s: HashSet<_> = cli.days.iter().collect();
        v.into_iter().filter(|(x, _)| s.contains(&x)).collect()
    } else if cli.all {
        v
    } else {
        vec![*v.last().unwrap()]
    }
}
