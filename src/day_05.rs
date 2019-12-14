use crate::intcode::{Computer, ComputerError, Value, Memory, parse_program};
use futures_await_test::*;
use anyhow::Result;

pub fn parse_input(s: &str) -> Result<Memory, ComputerError> {
    parse_program(s)
}

pub async fn part_1(values: Memory) -> Result<Value, ComputerError> {
    let mut output = Vec::new();
    let mut input: &[Value] = &[1];
    let mut comp = Computer::load(values);
    comp.set_input(Some(&mut input));
    comp.set_output(Some(&mut output));
    comp.run().await?;
    let last = output.pop().unwrap();
    assert!(output.iter().all(|i| *i == 0));
    Ok(last)
}

pub async fn part_2(values: Memory) -> Result<Value, ComputerError> {
    let mut output = None;
    let mut input: &[Value] = &[5];
    let mut comp = Computer::load(values);
    comp.set_input(Some(&mut input));
    comp.set_output(Some(&mut output));
    comp.run().await?;
    Ok(output.unwrap())
}

#[async_test]
async fn test_day_2_examples() -> Result<()> {
    let program = parse_program("1,0,0,0,99")?;
    let mut comp = Computer::load(program);
    comp.run().await?;
    assert_eq!(comp.base_memory(), &[2, 0, 0, 0, 99]);

    let program = parse_program("2,3,0,3,99")?;
    let mut comp = Computer::load(program);
    comp.run().await?;
    assert_eq!(comp.base_memory(), &[2, 3, 0, 6, 99]);

    let program = parse_program("2,4,4,5,99,0")?;
    let mut comp = Computer::load(program);
    comp.run().await?;
    assert_eq!(comp.base_memory(), &[2, 4, 4, 5, 99, 9801]);

    let program = parse_program("1,1,1,4,99,5,6,0,99")?;
    let mut comp = Computer::load(program);
    comp.run().await?;
    assert_eq!(comp.base_memory(), &[30, 1, 1, 4, 2, 5, 6, 0, 99]);
    
    Ok(())
}
