use crate::intcode::{Computer, ComputerError, Value};
use futures_await_test::*;

pub fn parse_input(s: &str) -> Vec<Value> {
    s.trim()
        .split(',')
        .map(|s| s.parse::<Value>().unwrap())
        .collect()
}

pub async fn part_1(values: Vec<Value>) -> Result<Value, ComputerError> {
    let mut output = Vec::new();
    let mut input: &[Value] = &[1];
    let mut comp = Computer::load(values);
    comp.run(&mut input, Some(&mut output)).await?;
    let last = output.pop().unwrap();
    assert!(output.iter().all(|i| *i == 0));
    Ok(last)
}

pub async fn part_2(values: Vec<Value>) -> Result<Value, ComputerError> {
    let mut output = &mut None;
    let mut input: &[Value] = &[5];
    let mut comp = Computer::load(values);
    comp.run(&mut input, Some(&mut output)).await?;
    Ok(output.unwrap())
}

#[async_test]
async fn test_day_2_examples() {
    let mut no_input: &[Value] = &[];

    let program = "1,0,0,0,99";
    let mut comp = Computer::load(parse_input(program));
    comp.run(&mut no_input, None).await.unwrap();
    assert_eq!(comp.memory(), &[2, 0, 0, 0, 99]);

    let program = "2,3,0,3,99";
    let mut comp = Computer::load(parse_input(program));
    comp.run(&mut no_input, None).await.unwrap();
    assert_eq!(comp.memory(), &[2, 3, 0, 6, 99]);

    let program = "2,4,4,5,99,0";
    let mut comp = Computer::load(parse_input(program));
    comp.run(&mut no_input, None).await.unwrap();
    assert_eq!(comp.memory(), &[2, 4, 4, 5, 99, 9801]);

    let program = "1,1,1,4,99,5,6,0,99";
    let mut comp = Computer::load(parse_input(program));
    comp.run(&mut no_input, None).await.unwrap();
    assert_eq!(comp.memory(), &[30, 1, 1, 4, 2, 5, 6, 0, 99]);
}
