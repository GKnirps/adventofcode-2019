use std::collections::{HashMap, VecDeque};
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_file(&Path::new(&filename)).map_err(|e| e.to_string())?;
    let lines: Vec<&str> = content.split('\n').collect();
    let reactions = parse_reactions(&lines)?;

    let ore_amount = ore_amount_for_fuel(&reactions, 1)?;
    println!("One unit fuel needs {} units of ore", ore_amount);

    let fuel_amount = fuel_amount_for_ore(&reactions, 1_000_000_000_000)?;
    println!(
        "A trillion units of ore will get us {} units of fuel.",
        fuel_amount
    );

    Ok(())
}

fn read_file(path: &Path) -> std::io::Result<String> {
    let ifile = File::open(path)?;
    let mut bufr = BufReader::new(ifile);
    let mut result = String::with_capacity(2048);
    bufr.read_to_string(&mut result)?;

    Ok(result)
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct Chemical<'a> {
    name: &'a str,
    amount: i64,
}

fn parse_chemical(s: &str) -> Result<Chemical, String> {
    let mut splitted = s.trim().split_whitespace();
    let amount: i64 = splitted
        .next()
        .ok_or_else(|| "Expected an amount".to_owned())?
        .parse::<i64>()
        .map_err(|e| e.to_string())?;
    let name = splitted.next().ok_or_else(|| "Expected a name")?;

    Ok(Chemical { name, amount })
}

type Reaction<'a> = (Vec<Chemical<'a>>, Chemical<'a>);

fn parse_reaction(line: &str) -> Result<Reaction, String> {
    let mut reaction_splitted = line.split("=>").map(|s| s.trim());
    let inputs: Vec<Chemical> = reaction_splitted
        .next()
        .ok_or_else(|| "expected input chemicals")?
        .trim()
        .split(',')
        .map(parse_chemical)
        .collect::<Result<Vec<Chemical>, String>>()?;
    let output = parse_chemical(
        reaction_splitted
            .next()
            .ok_or_else(|| "expected output chemical")?,
    )?;

    Ok((inputs, output))
}

fn parse_reactions<'a>(lines: &[&'a str]) -> Result<Vec<Reaction<'a>>, String> {
    lines
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(parse_reaction)
        .collect()
}

fn ore_amount_for_fuel(reactions: &[Reaction], fuel_amount: i64) -> Result<i64, String> {
    // assumption for this approach: reactions are a directed acyclic graph
    // I may have made some other assumptions here, but apparently, it works for my input.
    let reactions_by_output: HashMap<&str, &Reaction> = reactions
        .iter()
        .map(|reaction| (reaction.1.name, reaction))
        .collect();

    let mut materials_required: HashMap<&str, i64> = HashMap::with_capacity(reactions.len());
    materials_required.insert("FUEL", fuel_amount);
    let mut resolve_queue: VecDeque<&str> = VecDeque::with_capacity(reactions.len());
    resolve_queue.push_back("FUEL");

    while let Some(chemical_name) = resolve_queue.pop_front() {
        if chemical_name == "ORE" {
            continue;
        }
        if let Some(required_amount) = materials_required.get(chemical_name) {
            if *required_amount < 1 {
                continue;
            }
            let (inputs, output) = reactions_by_output
                .get(chemical_name)
                .ok_or_else(|| format!("There is no reaction that produces {}", chemical_name))?;
            let n_reactions = required_amount / output.amount
                + if required_amount % output.amount == 0 {
                    0
                } else {
                    1
                };
            for input in inputs {
                let n = materials_required.entry(input.name).or_insert(0);
                *n += input.amount * n_reactions;
                if *n > 0 {
                    resolve_queue.push_back(input.name);
                }
            }
            *materials_required.entry(chemical_name).or_insert(0) -= n_reactions * output.amount;
        }
    }

    materials_required
        .get("ORE")
        .cloned()
        .ok_or_else(|| "Apparently, we don't need no ore. This is probably wrong.".to_owned())
}

fn fuel_amount_for_ore(reactions: &[Reaction], ore_amount: i64) -> Result<i64, String> {
    // I am too lazy to think of an original way, so I will just brute force the solution with a binary search.

    // For that, we need a lower bound first
    let ore_for_one_fuel = ore_amount_for_fuel(reactions, 1)?;
    let mut lower_bound = ore_amount / ore_for_one_fuel;

    // then we search an upper bound in increasing steps
    let mut upper_bound = lower_bound * 2;
    while ore_amount >= ore_amount_for_fuel(reactions, upper_bound)? {
        upper_bound = upper_bound
            .checked_mul(2)
            .ok_or_else(|| "Unable to find upper bound in search range".to_owned())?;
    }

    // now we can do a binary search
    while lower_bound < upper_bound - 1 {
        let pivot = lower_bound + (upper_bound - lower_bound) / 2;
        if ore_amount < ore_amount_for_fuel(reactions, pivot)? {
            upper_bound = pivot;
        } else {
            lower_bound = pivot;
        }
    }
    Ok(lower_bound)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_valid_chemical() {
        // given
        let input = " 97   Au ";

        // when
        let result = parse_chemical(input).expect("Expected successful parsing");

        // then
        assert_eq!(
            result,
            Chemical {
                name: "Au",
                amount: 97
            }
        );
    }

    #[test]
    fn test_parse_invalid_chemical() {
        // given
        let input = "some gold";

        // when
        let result = parse_chemical(input);

        // then
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_valid_reaction() {
        // given
        let input = "  1 O,  2 H => 1 H2O ";

        // when
        let result = parse_reaction(input).expect("Expected successful parsing");

        // then
        assert_eq!(
            result,
            (
                vec![
                    Chemical {
                        name: "O",
                        amount: 1
                    },
                    Chemical {
                        name: "H",
                        amount: 2
                    }
                ],
                Chemical {
                    name: "H2O",
                    amount: 1
                }
            )
        );
    }

    #[test]
    fn test_puzzle1_example1() {
        // given
        let raw_reactions = &[
            "9 ORE => 2 A",
            "8 ORE => 3 B",
            "7 ORE => 5 C",
            "3 A, 4 B => 1 AB",
            "5 B, 7 C => 1 BC",
            "4 C, 1 A => 1 CA",
            "2 AB, 3 BC, 4 CA => 1 FUEL",
        ];
        let reactions = parse_reactions(raw_reactions).expect("Expected valid reactions");

        // when
        let ore_amount =
            ore_amount_for_fuel(&reactions, 1).expect("Expected to get an amount of ore");

        // then
        assert_eq!(ore_amount, 165);
    }
}
