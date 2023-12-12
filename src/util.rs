use crate::AOC_YEAR;
use anyhow::{Context, Result};
use directories::BaseDirs;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use std::env;
use std::fs;
use std::path::PathBuf;

pub struct InputSource {
    client: Client,
}

impl InputSource {
    pub fn new() -> Result<InputSource> {
        let sessionvar = format!("AOC{}_SESSION", AOC_YEAR);

        let session = env::var(&sessionvar).with_context(|| {
            format!(
                "Environment variable {sessionvar} is unset.
Set it to the value of the `session` cookie from the advent of code website."
            )
        })?;

        let mut headers = HeaderMap::new();
        let ck = HeaderValue::from_str(&format!("session={}", session))?;
        headers.insert("cookie", ck);

        let c = Client::builder().default_headers(headers).build()?;

        Ok(InputSource { client: c })
    }

    pub fn get(&self, day: usize) -> Result<String> {
        if let Some(s) = InputSource::get_cache(day) {
            return Ok(s);
        }

        let r = self.get_https(day)?;

        InputSource::put_cache(day, &r);

        Ok(r)
    }

    fn get_https(&self, day: usize) -> Result<String> {
        const AOC_YEAR_FULL: u32 = 2000 + AOC_YEAR;
        let url = format!(
            "https://adventofcode.com/{}/day/{}/input",
            AOC_YEAR_FULL, day
        );
        Ok(self.client.get(&url[..]).send()?.text()?)
    }

    fn cache_folder() -> String {
        format!("aoc{}", AOC_YEAR)
    }

    fn get_cache(day: usize) -> Option<String> {
        let base_dirs = BaseDirs::new()?;

        let mut path = PathBuf::new();
        path.push(base_dirs.cache_dir());
        path.push(Self::cache_folder());
        path.push(day.to_string());

        fs::read_to_string(&path).ok()
    }

    fn put_cache(day: usize, contents: &str) {
        let base_dirs = match BaseDirs::new() {
            Some(x) => x,
            None => {
                return;
            }
        };

        let mut path = PathBuf::new();
        path.push(base_dirs.cache_dir());
        path.push(Self::cache_folder());
        if !path.exists() {
            if let Err(err) = fs::create_dir(&path) {
                eprintln!("error creating cache dir {:?}: {}", path, err);
                return;
            }
        }
        path.push(day.to_string());
        if let Err(err) = fs::write(&path, &contents) {
            eprintln!("error writing cache file {:?}: {}", path, err);
        }
    }
}
