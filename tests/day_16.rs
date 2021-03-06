pub use adventofcode::day_16::{parse_input, part_1};
use anyhow::Result;
use futures_await_test::*;

#[async_test]
async fn test_part_1() -> Result<()> {
    use std::fs::File;
    use std::io::Read;
    let mut s = String::new();
    let mut file = File::open("input/day16")?;
    file.read_to_string(&mut s)?;
    assert_eq!(part_1(parse_input(&s)?), "19239468");
    Ok(())
}
