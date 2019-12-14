use crate::intcode::{parse_program, Computer, ComputerError, Memory, Value};

pub fn parse_input(s: &str) -> Result<Memory, ComputerError> {
    parse_program(s)
}

pub async fn part_1(mut input: Memory) -> Result<Value, ComputerError> {
    *input.get_mut(1) = 12;
    *input.get_mut(2) = 2;
    let mut computer = Computer::load(input);
    computer.run().await?;
    Ok(computer.memory().get(0))
}

pub async fn part_2(input: Memory) -> Result<Value, ComputerError> {
    for i in 0..100 {
        for j in 0..100 {
            let mut input = input.clone();
            *input.get_mut(1) = i;
            *input.get_mut(2) = j;
            let mut computer = Computer::load(input);
            computer.run().await?;
            if computer.memory().get(0) == 19_690_720 {
                return Ok(100 * i + j);
            }
        }
    }
    unreachable!();
}
