use std::collections::HashMap;

use anyhow::Result;

pub fn run(input: &str) -> Result<String> {
    Ok(format!("{} {}", p1(input), p2(input)))
}

fn p1(input: &str) -> u32 {
    nums(input)
        .iter()
        .filter_map(|num| num.sym.is_some().then_some(num.value))
        .sum()
}

fn p2(input: &str) -> u32 {
    let mut m = HashMap::<_, Vec<_>>::new();

    for num in nums(input) {
        if let Some(sym) = num.sym {
            if sym.c == b'*' {
                m.entry((sym.x, sym.y)).or_default().push(num.value);
            }
        }
    }

    m.values()
        .filter_map(|v| (v.len() == 2).then(|| v[0] * v[1]))
        .sum()
}

#[derive(Debug, Clone, Copy)]
struct Num {
    x: i16,
    y: i16,
    w: i16,

    sym: Option<Sym>,
    value: u32,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
struct Sym {
    x: i16,
    y: i16,
    c: u8,
}

fn nums(input: &str) -> Vec<Num> {
    let v: Vec<Vec<u8>> = input.lines().map(|l| l.bytes().collect()).collect();

    let mut nums = vec![];

    for (y, line) in v.iter().enumerate() {
        let mut acc: Option<Num> = None;
        for (x, dgt) in line.iter().map(|&c| (c as char).to_digit(10)).enumerate() {
            if let Some(dgt) = dgt {
                if let Some(acc) = acc.as_mut() {
                    acc.w += 1;
                    acc.value = acc.value * 10 + dgt;
                } else {
                    acc = Some(Num {
                        x: x as i16,
                        y: y as i16,
                        w: 1,
                        sym: None,
                        value: dgt,
                    })
                }
            } else if let Some(num) = acc.take() {
                nums.push(num)
            }
        }

        if let Some(num) = acc.take() {
            nums.push(num)
        }
    }

    let get_sym = |x, y| -> Option<Sym> {
        if x < 0 || y < 0 {
            return None;
        }
        let ys = y as usize;
        if ys >= v.len() {
            return None;
        }
        let line = &v[ys];
        let xs = x as usize;
        if xs >= line.len() {
            return None;
        }

        let c = line[xs];
        (c != b'.').then_some(Sym { x, y, c })
    };

    let get_num_sym = |num: &Num| -> Option<Sym> {
        let z = get_sym(num.x - 1, num.y).or_else(|| get_sym(num.x + num.w, num.y));
        if z.is_some() {
            return z;
        }

        for x in (num.x - 1)..=(num.x + num.w) {
            let z = get_sym(x, num.y - 1).or_else(|| get_sym(x, num.y + 1));
            if z.is_some() {
                return z;
            }
        }

        None
    };

    for num in nums.iter_mut() {
        num.sym = get_num_sym(num)
    }

    nums
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nums() {
        let src = "\
467..114..
...*......
..35..633.
......#...
617*......
.....+.58.
..592.....
......755.
...$.*....
.664.598..
";
        assert_eq!(p1(src), 4361);
        assert_eq!(p2(src), 467835);
    }
}
