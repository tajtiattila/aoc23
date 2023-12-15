use anyhow::Result;

pub fn run(input: &str) -> Result<String> {
    let p1 = input.trim().split(',').map(xhash).sum::<usize>();

    Ok(format!("{p1}"))
}

fn xhash(s: &str) -> usize {
    s.as_bytes()
        .iter()
        .fold(0, |acc, &c| ((acc + c as usize) * 17) % 256)
}
