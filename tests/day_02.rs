use adventofcode::day_02::*;
use anyhow::Result;
use std::fs::File;
use std::io::Read;
use futures_await_test::*;

#[async_test]
async fn test_part_1() -> Result<()> {
    let mut s = String::new();
    File::open("./input/day2")?.read_to_string(&mut s)?;
    assert_eq!(part_1(parse_input(&s)?).await?, 4330636);
    Ok(())
}

#[async_test]
async fn test_part_2() -> Result<()> {
    let mut s = String::new();
    File::open("./input/day2")?.read_to_string(&mut s)?;
    assert_eq!(part_2(parse_input(&s)?).await?, 6086);
    Ok(())
}
