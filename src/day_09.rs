use crate::intcode::{parse_program, Computer, ComputerError, Memory, Value};

pub fn parse_input(s: &str) -> Result<Memory, ComputerError> {
    parse_program(s)
}

pub async fn part_1(values: Memory) -> Result<Value, ComputerError> {
    let mut output = None;
    let mut input: &[Value] = &[1];
    let mut comp = Computer::load(values);
    comp.set_input(Some(&mut input));
    comp.set_output(Some(&mut output));
    comp.run().await?;
    Ok(output.unwrap())
}

pub async fn part_2(values: Memory) -> Result<Value, ComputerError> {
    let mut output = None;
    let mut input: &[Value] = &[2];
    let mut comp = Computer::load(values);
    comp.set_input(Some(&mut input));
    comp.set_output(Some(&mut output));
    comp.run().await?;
    Ok(output.unwrap())
}
