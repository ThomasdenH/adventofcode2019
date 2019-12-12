use adventofcode::day_03::*;
use adventofcode::Result;
use std::fs::File;
use std::io::Read;

#[test]
fn test_part_1() -> Result<()> {
    let mut s = String::new();
    File::open("./input/day3")?.read_to_string(&mut s)?;
    assert_eq!(part_1(parse_input(&s)), 1195);
    Ok(())
}

#[test]
fn test_part_2() -> Result<()> {
    let mut s = String::new();
    File::open("./input/day3")?.read_to_string(&mut s)?;
    assert_eq!(part_2(parse_input(&s)), 91518);
    Ok(())
}
