use anyhow::Result;
use clap::Parser;
use once_cell::sync::OnceCell;
use std::collections::HashSet;
use std::time::{Duration, Instant};

const AOC_YEAR: u32 = 23;

mod grid;

// static_mod_funcs creates a static slice of `name` that
// contains the methods `mname` as `mty` in the specified modules `m`.
macro_rules! static_mod_funcs {
    ( $name:ident, $mname:ident as $mty:ty, [ $( $m:ident ),* ] ) => {
        $(
            mod $m;
        )*
        static $name: &[$mty] = &[
            $(
                ($m::$mname as $mty),
            )*
        ];
    }
}

static_mod_funcs!(
    DAY_FNS,
    run as DayFunc,
    [
        day01, day02, day03, day04, day05, day06, day07, day08, day09, day10, day11, day12, day13,
        day14, day15, day16, day17
    ]
);

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
    let v: Vec<(usize, DayFunc)> = DAY_FNS
        .iter()
        .enumerate()
        .map(|(n, &f)| (n + 1, f))
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
