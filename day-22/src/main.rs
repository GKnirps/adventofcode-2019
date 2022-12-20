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

    let index = apply_techniques_to_index(&techniques, 2019, 10007);
    println!("After shuffling one time, card 2019 is at index: {}", index);

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
    Cut(isize),
    DealWithInc(usize),
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
        let cut: isize = cut_substr.parse::<isize>().map_err(|e| e.to_string())?;
        Ok(Technique::Cut(cut))
    } else if line.starts_with(DEAL_WITH_INCREMENT_NAME)
        && line.len() > DEAL_WITH_INCREMENT_NAME.len()
    {
        let (_, increment_substr) = line.split_at(DEAL_WITH_INCREMENT_NAME.len());
        let increment: usize = increment_substr
            .parse::<usize>()
            .map_err(|e| e.to_string())?;
        Ok(Technique::DealWithInc(increment))
    } else {
        Err("Unknown technique".to_owned())
    }
}

fn apply_techniques_to_index(techniques: &[Technique], index: usize, stack_size: usize) -> usize {
    techniques.iter().fold(index, |perm, tech| {
        apply_technique_to_index(*tech, perm, stack_size)
    })
}

fn apply_technique_to_index(technique: Technique, index: usize, stack_size: usize) -> usize {
    match technique {
        Technique::DealIntoNew => stack_size - 1 - index,
        Technique::Cut(i) => cut_index(index, stack_size, i),
        Technique::DealWithInc(n) => (index * n) % stack_size,
    }
}

fn cut_index(index: usize, stack_size: usize, icutoff: isize) -> usize {
    let cutoff: usize = (icutoff % stack_size as isize + stack_size as isize) as usize % stack_size;
    if index < cutoff {
        index + (stack_size - cutoff)
    } else {
        index - cutoff
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
    fn test_cut() {
        assert_eq!(
            cut(vec!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9), 3),
            vec!(3, 4, 5, 6, 7, 8, 9, 0, 1, 2)
        );
        assert_eq!(
            cut(vec!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9), -4),
            vec!(6, 7, 8, 9, 0, 1, 2, 3, 4, 5)
        );
    }

    #[test]
    fn test_deal_with_increment() {
        assert_eq!(
            deal_with_increment(vec!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9), 3),
            vec!(0, 7, 4, 1, 8, 5, 2, 9, 6, 3)
        );
    }

    #[test]
    fn test_example_4() {
        // given
        let lines = &[
            "deal into new stack",
            "cut -2",
            "deal with increment 7",
            "cut 8",
            "cut -4",
            "deal with increment 7",
            "cut 3",
            "deal with increment 9",
            "deal with increment 3",
            "cut -1",
        ];
        let techniques = parse_techniques(lines).expect("Expected valid techniques");

        let input: Vec<usize> = (0..10).collect();

        // when
        let result = apply_techniques(&techniques, input);

        // then
        assert_eq!(&result, &[9, 2, 5, 8, 1, 4, 7, 0, 3, 6]);
    }
}
