fn main() -> Result<(), String> {
    // I hardcoded my inputs. Deal with it.
    let lower_bound: u32 = 145_852;
    let upper_bound: u32 = 616_942;

    let possible_keys = count_possible_keys(lower_bound, upper_bound);
    println!(
        "There are {} possible keys in the range [{}, {}]",
        possible_keys, lower_bound, upper_bound
    );

    Ok(())
}

fn key_matches(key: u32) -> bool {
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

fn count_possible_keys(lower: u32, upper: u32) -> usize {
    (lower..=upper).filter(|key| key_matches(*key)).count()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn key_matches_works_for_examples() {
        // positiv examples
        assert!(key_matches(122_345));
        assert!(key_matches(111_111));

        // negative examples
        assert!(!key_matches(223_450));
        assert!(!key_matches(123_789));
    }
}
