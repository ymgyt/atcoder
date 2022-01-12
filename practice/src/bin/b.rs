use proconio::{input, source::line::LineSource};
use std::{char, cmp::Ordering, collections::HashMap, io};

fn insert(
    mut vs: Vec<char>,
    current: char,
    compare: &mut impl FnMut(char, char) -> Ordering,
) -> Vec<char> {
    if vs.is_empty() {
        return vec![current];
    }

    let (mut start, mut end) = (0, vs.len());

    while start < end {
        let middle = (start + end) / 2;
        match compare(vs[middle], current) {
            Ordering::Less => {
                start = middle + 1;
            }
            Ordering::Greater => {
                end = middle;
            }
            _ => unreachable!(),
        }
    }

    vs.insert(start, current);
    vs
}

fn main() {
    let mut stdin = LineSource::new(io::BufReader::new(io::stdin()));

    input! (
        from &mut stdin,
        n: usize,
        _: usize,
    );

    let mut cache = HashMap::new();

    let mut compare = |x: char, y: char| -> Ordering {
        if let Some(&hit) = cache.get(&(x, y)) {
            return hit;
        }

        println!("? {} {}", x, y);
        input!(
            from &mut stdin,
            c: char,
        );
        let result = match c {
            '>' => Ordering::Greater,
            '<' => Ordering::Less,
            _ => unreachable!(),
        };

        cache.insert((x, y), result);
        result
    };

    let vs: Vec<char> = (0..n)
        .into_iter()
        .enumerate()
        .map(|(idx, _)| char::from_u32(idx as u32 + 65).unwrap())
        .fold(Vec::new(), |vs, current| insert(vs, current, &mut compare));

    println!("! {}", vs.into_iter().collect::<String>());
}
