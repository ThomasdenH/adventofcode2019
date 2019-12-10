use adventofcode::day5::*;
use adventofcode::Result;
use futures_await_test::async_test;
use std::fs::File;
use std::io::Read;

#[async_test]
async fn test_part_1() -> Result<()> {
    let mut s = String::new();
    File::open("./input/day5")?.read_to_string(&mut s)?;
    assert_eq!(part_1(parse_input(&s)).await?, 5044655);
    Ok(())
}

#[async_test]
async fn test_part_2() -> Result<()> {
    let mut s = String::new();
    File::open("./input/day5")?.read_to_string(&mut s)?;
    assert_eq!(part_2(parse_input(&s)).await?, 7408802);
    Ok(())
}
