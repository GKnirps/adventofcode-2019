fn main() -> Result<(), String> {
    // I hardcoded my inputs. Deal with it.
    let lower_bound: u32 = 145_852;
    let upper_bound: u32 = 616_942;

    let possible_keys_p1 = count_possible_keys_puzzle1(lower_bound, upper_bound);
    println!(
        "There are {} possible keys in the range [{}, {}]",
        possible_keys_p1, lower_bound, upper_bound
    );

    let possible_keys_p2 = count_possible_keys_puzzle2(lower_bound, upper_bound);
    println!(
        "There are {} possible keys for the updated rules in the range[{}, {}]",
        possible_keys_p2, lower_bound, upper_bound
    );

    Ok(())
}

fn key_matches_puzzle1(key: u32) -> bool {
    if key < 100_000 || key > 999_999 {
        return false;
    }
    let mut prev = key % 10;
    let mut unprocessed = key / 10;
    let mut found_twins = false;

    while unprocessed > 0 {
        let current = unprocessed % 10;
        if current > prev {
            return false;
        }
        found_twins = found_twins || current == prev;

        prev = current;
        unprocessed /= 10;
    }

    found_twins
}

fn count_possible_keys_puzzle1(lower: u32, upper: u32) -> usize {
    (lower..=upper)
        .filter(|key| key_matches_puzzle1(*key))
        .count()
}

fn key_matches_puzzle2(key: u32) -> bool {
    if key < 100_000 || key > 999_999 {
        return false;
    }
    let mut prev = key % 10;
    let mut unprocessed = key / 10;
    let mut found_twins = false;
    let mut tuple_count = 1;

    while unprocessed > 0 {
        let current = unprocessed % 10;
        if current > prev {
            return false;
        }
        if prev == current {
            tuple_count += 1;
        } else {
            found_twins = found_twins || tuple_count == 2;
            tuple_count = 1;
        }

        prev = current;
        unprocessed /= 10;
    }

    found_twins || tuple_count == 2
}

fn count_possible_keys_puzzle2(lower: u32, upper: u32) -> usize {
    (lower..=upper)
        .filter(|key| key_matches_puzzle2(*key))
        .count()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn key_matches_puzzle1_works_for_examples() {
        // positiv examples
        assert!(key_matches_puzzle1(122_345));
        assert!(key_matches_puzzle1(111_111));

        // negative examples
        assert!(!key_matches_puzzle1(223_450));
        assert!(!key_matches_puzzle1(123_789));
    }

    #[test]
    fn key_matches_puzzle2_works_for_examples() {
        // positive examples
        assert!(key_matches_puzzle2(122_345));
        assert!(key_matches_puzzle2(112_233));
        assert!(key_matches_puzzle2(111_122));
        assert!(key_matches_puzzle2(112_222));

        // negative examples
        assert!(!key_matches_puzzle2(123_444));
        assert!(!key_matches_puzzle2(223_450));
        assert!(!key_matches_puzzle2(111_111));
        assert!(!key_matches_puzzle2(123_789));
    }
}
