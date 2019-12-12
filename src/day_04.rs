use itertools::Itertools;

pub fn parse_input(input: &str) -> Vec<u32> {
    input
        .split('-')
        .map(|s| s.parse::<u32>().unwrap())
        .collect()
}

fn digits(mut u: u32) -> impl Iterator<Item = u32> + 'static {
    std::iter::from_fn(move || {
        if u > 0 {
            let d = u % 10;
            u /= 10;
            Some(d)
        } else {
            None
        }
    })
}

fn increasing(u: u32) -> bool {
    digits(u).zip(digits(u).skip(1)).all(|(d1, d2)| d1 >= d2)
}

fn has_duplicates(u: u32) -> bool {
    digits(u)
        .group_by(|u| *u)
        .into_iter()
        .map(|(_, g)| g.count())
        .any(|total| total >= 2)
}

fn meets_criteria(u: u32) -> bool {
    u <= 999_999 && increasing(u) && has_duplicates(u)
}

fn has_streak_of_two(u: u32) -> bool {
    digits(u)
        .group_by(|u| *u)
        .into_iter()
        .map(|(_, g)| g.count())
        .any(|total| total == 2)
}

fn meets_criteria_part_2(u: u32) -> bool {
    u <= 999_999 && increasing(u) && has_streak_of_two(u)
}

pub fn part_1(input: Vec<u32>) -> usize {
    let start = input[0];
    let end = input[1];
    (start..=end).filter(|&u| meets_criteria(u)).count()
}

pub fn part_2(input: Vec<u32>) -> usize {
    let start = input[0];
    let end = input[1];
    (start..=end).filter(|&u| meets_criteria_part_2(u)).count()
}

#[test]
fn test_examples() {
    assert!(meets_criteria(111111));
    assert!(!meets_criteria(223450));
    assert!(!meets_criteria(123789));
}

#[test]
fn test_examples_part_2() {
    assert!(meets_criteria_part_2(112233));
    assert!(!meets_criteria_part_2(123444));
    assert!(meets_criteria_part_2(111122));
}
