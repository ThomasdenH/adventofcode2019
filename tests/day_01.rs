use adventofcode::day_01::*;
use anyhow::Result;
use std::fs::File;
use std::io::Read;

#[test]
fn test_part_1() -> Result<()> {
    let mut s = String::new();
    File::open("./input/day1")?.read_to_string(&mut s)?;
    assert_eq!(part_1(parse_input(&s)?.into_iter()), 3421505);
    Ok(())
}

#[test]
fn test_part_2() -> Result<()> {
    let mut s = String::new();
    File::open("./input/day1")?.read_to_string(&mut s)?;
    assert_eq!(part_2(parse_input(&s)?.into_iter()), 5129386);
    Ok(())
}
