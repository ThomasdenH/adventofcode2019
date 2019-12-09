use std::convert::TryFrom;
use std::iter;
use thiserror::*;

pub type Value = isize;

pub trait Read {
    fn read(&mut self) -> Option<Value>;
}

impl<I: Iterator<Item = Value>> Read for I {
    fn read(&mut self) -> Option<Value> {
        self.next()
    }
}

pub trait Write {
    fn write(&mut self, output: Value);
}

impl Write for Vec<Value> {
    fn write(&mut self, output: Value) {
        self.push(output)
    }
}

/// Unwraps items from an iterator automatically or returns E.
struct IteratorOkOrRepeat<Item, I: Iterator<Item = Result<Item, E>>, E: Clone> {
    iterator: I,
    error: E,
}

impl<Item, I: Iterator<Item = Result<Item, E>>, E: Clone> IteratorOkOrRepeat<Item, I, E> {
    fn next(&mut self) -> Result<Item, E> {
        if let Some(n) = self.iterator.next() {
            n
        } else {
            Err(self.error.clone())
        }
    }
}

trait IteratorOkOrRepeatable<Item, I: Iterator<Item = Result<Item, E>>, E: Clone> {
    fn ok_or_repeat(self, error: E) -> IteratorOkOrRepeat<Item, I, E>;
}

impl<Item, I: Iterator<Item = Result<Item, E>>, E: Clone> IteratorOkOrRepeatable<Item, I, E> for I {
    fn ok_or_repeat(self, error: E) -> IteratorOkOrRepeat<Item, I, E> {
        IteratorOkOrRepeat {
            iterator: self,
            error,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum OpCode {
    Add,
    Multiply,
    Input,
    Output,
    JumpIfTrue,
    JumpIfFalse,
    LessThan,
    Equals,
    Quit,
}

#[derive(Clone, Copy, Debug)]
enum ParameterMode {
    Position,
    Immediate,
}

#[derive(Error, Debug, Clone, Copy)]
pub enum ComputerError {
    #[error("unknown op code: {0}")]
    UnknownOpCode(Value),
    #[error("unknown parameter mode: {0}")]
    UnknownParameterMode(Value),
    #[error("parameter write in immediate mode")]
    WriteInImmediateMode,
    #[error("expected parameter, but the memory stops here")]
    ExpectedParameter,
    #[error("expected input")]
    ReadInputError,
    #[error("attempted to write outside of memory")]
    WriteOutsideOfMemory,
    #[error("attempted to read outside of memory")]
    ReadOutsideOfMemory,
}

impl TryFrom<Value> for OpCode {
    type Error = ComputerError;
    fn try_from(u: Value) -> Result<Self, Self::Error> {
        match u {
            1 => Ok(OpCode::Add),
            2 => Ok(OpCode::Multiply),
            3 => Ok(OpCode::Input),
            4 => Ok(OpCode::Output),
            5 => Ok(OpCode::JumpIfTrue),
            6 => Ok(OpCode::JumpIfFalse),
            7 => Ok(OpCode::LessThan),
            8 => Ok(OpCode::Equals),
            99 => Ok(OpCode::Quit),
            other => Err(ComputerError::UnknownOpCode(other)),
        }
    }
}

impl TryFrom<Value> for ParameterMode {
    type Error = ComputerError;
    fn try_from(u: Value) -> Result<Self, Self::Error> {
        match u {
            0 => Ok(ParameterMode::Position),
            1 => Ok(ParameterMode::Immediate),
            other => Err(ComputerError::UnknownParameterMode(other)),
        }
    }
}

#[derive(Copy, Clone)]
struct Instruction(Value);

#[derive(Copy, Clone)]
struct ModesIterator(Value);

impl Iterator for ModesIterator {
    type Item = Result<ParameterMode, ComputerError>;
    fn next(&mut self) -> Option<Result<ParameterMode, ComputerError>> {
        let out = ParameterMode::try_from(self.0 % 10);
        self.0 /= 10;
        Some(out)
    }
}

impl Instruction {
    fn op_code(self) -> Result<OpCode, ComputerError> {
        OpCode::try_from(self.0 % 100)
    }

    fn modes(self) -> impl Iterator<Item = Result<ParameterMode, ComputerError>> {
        ModesIterator(self.0 / 100)
    }
}

pub struct Computer {
    memory: Vec<Value>,
    instruction_pointer: Value,
}

impl Computer {
    pub fn load(memory: Vec<Value>) -> Computer {
        Computer {
            memory,
            instruction_pointer: 0,
        }
    }

    fn advance_pointer(&mut self) -> Option<Value> {
        let val = self.memory.get(self.instruction_pointer as usize).copied();
        self.instruction_pointer += 1;
        val
    }

    pub fn run(
        &mut self,
        input: &mut dyn Read,
        mut output: Option<&mut dyn Write>,
    ) -> Result<(), ComputerError> {
        while let Some(instruction_value) = self.advance_pointer() {
            let instruction = Instruction(instruction_value);
            let mut parameters = instruction
                .modes()
                .zip(iter::from_fn(|| self.advance_pointer()))
                .map(|(a, b)| Ok((a?, b)))
                .ok_or_repeat(ComputerError::ExpectedParameter);
            match instruction.op_code()? {
                OpCode::Quit => return Ok(()),
                OpCode::Add => {
                    let a_at = parameters.next()?;
                    let b_at = parameters.next()?;
                    let to_at = parameters.next()?;
                    let a = self.get_parameter(a_at)?;
                    let b = self.get_parameter(b_at)?;
                    let to = self.get_parameter_mut(to_at)?;
                    *to = a + b;
                }
                OpCode::Multiply => {
                    let a_at = parameters.next()?;
                    let b_at = parameters.next()?;
                    let to_at = parameters.next()?;
                    let a = self.get_parameter(a_at)?;
                    let b = self.get_parameter(b_at)?;
                    let to = self.get_parameter_mut(to_at)?;
                    *to = a * b;
                }
                OpCode::Input => {
                    let value = input.read().ok_or(ComputerError::ReadInputError)?;
                    let to_at = parameters.next()?;
                    let to = self.get_parameter_mut(to_at)?;
                    *to = value;
                }
                OpCode::Output => {
                    let from_at = parameters.next()?;
                    let from = self.get_parameter(from_at)?;
                    if let Some(ref mut output) = output {
                        output.write(from);
                    }
                }
                OpCode::JumpIfTrue => {
                    let a_at = parameters.next()?;
                    let b_at = parameters.next()?;
                    let a = self.get_parameter(a_at)?;
                    let b = self.get_parameter(b_at)?;
                    if a != 0 {
                        self.instruction_pointer = b;
                    }
                }
                OpCode::JumpIfFalse => {
                    let a_at = parameters.next()?;
                    let b_at = parameters.next()?;
                    let a = self.get_parameter(a_at)?;
                    let b = self.get_parameter(b_at)?;
                    if a == 0 {
                        self.instruction_pointer = b;
                    }
                }
                OpCode::LessThan => {
                    let a_at = parameters.next()?;
                    let b_at = parameters.next()?;
                    let c_at = parameters.next()?;
                    let a = self.get_parameter(a_at)?;
                    let b = self.get_parameter(b_at)?;
                    let c = self.get_parameter_mut(c_at)?;
                    *c = if a < b { 1 } else { 0 };
                }
                OpCode::Equals => {
                    let a_at = parameters.next()?;
                    let b_at = parameters.next()?;
                    let c_at = parameters.next()?;
                    let a = self.get_parameter(a_at)?;
                    let b = self.get_parameter(b_at)?;
                    let c = self.get_parameter_mut(c_at)?;
                    *c = if a == b { 1 } else { 0 };
                }
            }
        }
        Ok(())
    }

    fn get_parameter(&self, (mode, pos): (ParameterMode, Value)) -> Result<Value, ComputerError> {
        match mode {
            ParameterMode::Immediate => Ok(pos),
            ParameterMode::Position => {
                let pos = usize::try_from(pos).map_err(|_| ComputerError::ReadOutsideOfMemory)?;
                self.memory
                    .get(pos)
                    .copied()
                    .ok_or(ComputerError::ReadOutsideOfMemory)
            }
        }
    }

    fn get_parameter_mut(
        &mut self,
        (mode, pos): (ParameterMode, Value),
    ) -> Result<&mut Value, ComputerError> {
        match mode {
            ParameterMode::Immediate => Err(ComputerError::WriteInImmediateMode),
            ParameterMode::Position => {
                let pos = usize::try_from(pos).map_err(|_| ComputerError::ReadOutsideOfMemory)?;
                self.memory
                    .get_mut(pos)
                    .ok_or(ComputerError::ReadOutsideOfMemory)
            }
        }
    }

    pub fn memory(&self) -> &[Value] {
        &self.memory
    }
}
