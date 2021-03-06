use adventofcode::day_07::{parse_input, part_1, part_2};
use anyhow::Result;
use futures_await_test::async_test;

#[async_test]
async fn test_part_1() -> Result<()> {
    use std::fs::File;
    use std::io::Read;
    let mut s = String::new();
    let mut file = File::open("input/day7")?;
    file.read_to_string(&mut s)?;
    assert_eq!(part_1(parse_input(&s)?).await?, 38500);
    Ok(())
}

#[async_test]
async fn test_part_2() -> Result<()> {
    use std::fs::File;
    use std::io::Read;
    let mut s = String::new();
    let mut file = File::open("input/day7")?;
    file.read_to_string(&mut s)?;
    assert_eq!(part_2(&parse_input(&s)?).await?, 33660560);
    Ok(())
}
