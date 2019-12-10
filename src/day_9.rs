use crate::intcode::{Computer, ComputerError, Value};

pub fn parse_input(s: &str) -> Vec<Value> {
    s.trim()
        .split(',')
        .map(|s| s.parse::<Value>().unwrap())
        .collect()
}

pub async fn part_1(values: Vec<Value>) -> Result<Value, ComputerError> {
    let mut output = &mut None;
    let mut input: &[Value] = &[1];
    let mut comp = Computer::load(values);
    comp.run(&mut input, Some(&mut output)).await?;
    Ok(output.unwrap())
}

pub async fn part_2(values: Vec<Value>) -> Result<Value, ComputerError> {
    let mut output = &mut None;
    let mut input: &[Value] = &[2];
    let mut comp = Computer::load(values);
    comp.run(&mut input, Some(&mut output)).await?;
    Ok(output.unwrap())
}
