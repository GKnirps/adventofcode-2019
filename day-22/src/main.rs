use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_file(Path::new(&filename)).map_err(|e| e.to_string())?;
    let lines: Vec<&str> = content.split('\n').collect();
    let techniques: Vec<Technique> = parse_techniques(&lines)?;

    let joined_technique_10007 = join_techniques(&techniques, 10007);
    let index = apply_joined_technique_to_index(joined_technique_10007, 2019, 10007);
    println!("After shuffling one time, card 2019 is at index: {index}");

    let large_stack_size: i64 = 119_315_717_514_047;
    let shuffle_rounds: i64 = 101_741_582_076_661;

    let end_index: i64 = 2020;

    let joined_technique_large = join_techniques(&techniques, large_stack_size);
    let reverted = reverse_index_a_ridiculous_number_of_times(
        joined_technique_large,
        end_index,
        large_stack_size,
        shuffle_rounds,
    );
    println!("The start index was {reverted}");

    Ok(())
}

fn read_file(path: &Path) -> std::io::Result<String> {
    let ifile = File::open(path)?;
    let mut bufr = BufReader::new(ifile);
    let mut result = String::with_capacity(2048);
    bufr.read_to_string(&mut result)?;

    Ok(result)
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
enum Technique {
    DealIntoNew,
    Cut(i64),
    DealWithInc(i64),
}

fn parse_techniques(lines: &[&str]) -> Result<Vec<Technique>, String> {
    lines
        .iter()
        .filter(|l| !l.is_empty())
        .map(|l| parse_technique(l))
        .collect()
}

static DEAL_INTO_NEW_NAME: &str = "deal into new stack";
static CUT_NAME: &str = "cut ";
static DEAL_WITH_INCREMENT_NAME: &str = "deal with increment ";
fn parse_technique(line: &str) -> Result<Technique, String> {
    if line == DEAL_INTO_NEW_NAME {
        Ok(Technique::DealIntoNew)
    } else if line.starts_with(CUT_NAME) && line.len() > CUT_NAME.len() {
        let (_, cut_substr) = line.split_at(CUT_NAME.len());
        let cut: i64 = cut_substr.parse::<i64>().map_err(|e| e.to_string())?;
        Ok(Technique::Cut(cut))
    } else if line.starts_with(DEAL_WITH_INCREMENT_NAME)
        && line.len() > DEAL_WITH_INCREMENT_NAME.len()
    {
        let (_, increment_substr) = line.split_at(DEAL_WITH_INCREMENT_NAME.len());
        let increment: i64 = increment_substr.parse::<i64>().map_err(|e| e.to_string())?;
        Ok(Technique::DealWithInc(increment))
    } else {
        Err("Unknown technique".to_owned())
    }
}

fn apply_joined_technique_to_index(joined: JoinedTechnique, index: i64, stack_size: i64) -> i64 {
    (index * joined.a + joined.b).rem_euclid(stack_size)
}

// represents index shuffle of the form (i*a + b) % stack_size
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct JoinedTechnique {
    a: i64,
    b: i64,
}

// this works fine if the stack size is a prime number (which they are for both part 1 and 2)
fn join_techniques(techniques: &[Technique], stack_size: i64) -> JoinedTechnique {
    techniques
        .iter()
        .map(|technique| match technique {
            Technique::DealIntoNew => JoinedTechnique {
                a: -1,
                b: stack_size - 1,
            },
            Technique::Cut(i) => JoinedTechnique { a: 1, b: -i },
            Technique::DealWithInc(n) => JoinedTechnique { a: *n, b: 0 },
        })
        .fold(
            JoinedTechnique { a: 1, b: 0 },
            |JoinedTechnique { a: a1, b: b1 }, JoinedTechnique { a: a2, b: b2 }| JoinedTechnique {
                a: (a1 * a2) % stack_size,
                b: (a2 * b1 + b2) % stack_size,
            },
        )
}

fn reverse_index_a_ridiculous_number_of_times(
    technique: JoinedTechnique,
    end_index: i64,
    stack_size: i64,
    shuffle_rounds: i64,
) -> i64 {
    // I had to look up a lot of modular arithmetic on wikipedia. I learned all of it at some
    // point, but I forgot most of it.
    // Of course, this only works if stack_size is a prime number, but fortunately, this is the
    // case.
    let am = pow_mod(
        technique.a as i128,
        shuffle_rounds as i128,
        stack_size as i128,
    ) as i64;
    let (_, _, am_inv) = extended_euclid(stack_size, am);
    let (_, _, d) = extended_euclid(stack_size, (1 - technique.a).rem_euclid(stack_size));
    let b = (technique.b as i128
        * ((1 - am).rem_euclid(stack_size) as i128 * d as i128).rem_euclid(stack_size as i128))
    .rem_euclid(stack_size as i128) as i64;
    ((end_index as i128 - b as i128).rem_euclid(stack_size as i128) * am_inv as i128)
        .rem_euclid(stack_size as i128) as i64
}

fn pow_mod(mut base: i128, mut exp: i128, m: i128) -> i128 {
    if m == 1 {
        return 0;
    }
    let mut result = 1;
    base %= m;
    while exp > 0 {
        if exp % 2 == 1 {
            result = result * base % m;
        }
        exp >>= 1;
        base = base * base % m
    }
    result
}

fn extended_euclid(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 {
        (a, 1, 0)
    } else {
        let (d, s, t) = extended_euclid(b, a % b);
        (d, t, s - (a / b) * t)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_technique() {
        assert_eq!(
            parse_technique("deal into new stack"),
            Ok(Technique::DealIntoNew)
        );
        assert_eq!(parse_technique("cut 42"), Ok(Technique::Cut(42)));
        assert_eq!(parse_technique("cut -42"), Ok(Technique::Cut(-42)));
        assert_eq!(
            parse_technique("deal with increment 42"),
            Ok(Technique::DealWithInc(42))
        );

        assert!(parse_technique("throw cards in the air").is_err());
        assert!(parse_technique("cut ").is_err());
        assert!(parse_technique("ceal with increment ").is_err());
    }

    #[test]
    fn pow_mod_works_correctly() {
        assert_eq!(pow_mod(3, 4, 11), 4);
    }
}
