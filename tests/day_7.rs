use adventofcode::day_7::Solution;
use anyhow::Result;
use aoc_types::DaySolution;

#[test]
fn test_part_1() -> Result<()> {
    use std::fs::File;
    use std::io::Read;
    let mut s = String::new();
    let mut file = File::open("input/day7")?;
    file.read_to_string(&mut s)?;
    assert_eq!(Solution::part_1(&Solution::parse_input(&s)?)?, 38500);
    Ok(())
}
