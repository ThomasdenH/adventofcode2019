use async_trait::async_trait;
use futures::channel::mpsc::{Receiver, Sender};
use futures::prelude::*;
use futures_await_test::async_test;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::iter;
use thiserror::*;

pub type Value = isize;

pub fn parse_program(s: &str) -> Vec<Value> {
    s.trim()
        .split(',')
        .map(|s| s.parse::<Value>().unwrap())
        .collect()
}

pub struct Memory {
    base: Vec<Value>,
    additional: HashMap<usize, Value>,
}

impl From<Vec<Value>> for Memory {
    fn from(base: Vec<Value>) -> Self {
        Memory {
            base,
            additional: HashMap::new(),
        }
    }
}

impl Memory {
    fn get(&self, pos: usize) -> Value {
        *self
            .base
            .get(pos)
            .unwrap_or_else(|| self.additional.get(&pos).unwrap_or(&0))
    }

    fn get_mut(&mut self, pos: usize) -> &mut Value {
        if let Some(val) = self.base.get_mut(pos) {
            val
        } else {
            self.additional.entry(pos).or_insert(0)
        }
    }
}

#[async_trait]
pub trait Read {
    async fn read(&mut self) -> Option<Value>;
}

#[async_trait]
impl Read for &'_ [Value] {
    async fn read(&mut self) -> Option<Value> {
        if let Some((value, remainder)) = self.split_first() {
            *self = remainder;
            Some(*value)
        } else {
            None
        }
    }
}

#[async_test]
async fn test_slice_input() {
    let mut input: &[Value] = &[0, 1, 2];
    assert_eq!(input.read().await, Some(0));
    assert_eq!(input.read().await, Some(1));
    assert_eq!(input.read().await, Some(2));
    assert_eq!(input.read().await, None);
}

#[async_trait]
impl Read for Receiver<Value> {
    async fn read(&mut self) -> Option<Value> {
        self.next().await
    }
}

#[async_trait]
pub trait Write {
    async fn write(&mut self, output: Value);
}

#[async_trait]
impl Write for Vec<Value> {
    async fn write(&mut self, output: Value) {
        self.push(output)
    }
}

#[async_trait]
impl Write for &'_ mut Option<Value> {
    async fn write(&mut self, output: Value) {
        **self = Some(output);
    }
}

#[async_trait]
impl Write for Sender<Value> {
    async fn write(&mut self, output: Value) {
        self.send(output).await.unwrap();
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
    RelativeBaseOffset,
    Quit,
}

#[derive(Clone, Copy, Debug)]
enum ParameterMode {
    Position,
    Immediate,
    Relative,
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
            9 => Ok(OpCode::RelativeBaseOffset),
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
            2 => Ok(ParameterMode::Relative),
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
    memory: Memory,
    instruction_pointer: Value,
    relative_base: Value,
}

impl Computer {
    pub fn load(memory: Vec<Value>) -> Computer {
        Computer {
            memory: Memory::from(memory),
            instruction_pointer: 0,
            relative_base: 0,
        }
    }

    fn advance_pointer(&mut self) -> Value {
        let val = self.memory.get(self.instruction_pointer as usize);
        self.instruction_pointer += 1;
        val
    }

    pub async fn run(
        &mut self,
        input: &mut dyn Read,
        mut output: Option<&mut dyn Write>,
    ) -> Result<(), ComputerError> {
        loop {
            let instruction_value = self.advance_pointer();
            let instruction = Instruction(instruction_value);
            let mut parameters = instruction
                .modes()
                .zip(iter::from_fn(|| Some(self.advance_pointer())))
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
                    let value = input.read().await.ok_or(ComputerError::ReadInputError)?;
                    let to_at = parameters.next()?;
                    let to = self.get_parameter_mut(to_at)?;
                    *to = value;
                }
                OpCode::Output => {
                    let from_at = parameters.next()?;
                    let from = self.get_parameter(from_at)?;
                    if let Some(ref mut output) = output {
                        output.write(from).await;
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
                OpCode::RelativeBaseOffset => {
                    let at = parameters.next()?;
                    let a = self.get_parameter(at)?;
                    self.relative_base += a;
                }
            }
        }
    }

    fn get_parameter(&self, (mode, pos): (ParameterMode, Value)) -> Result<Value, ComputerError> {
        match mode {
            ParameterMode::Immediate => Ok(pos),
            ParameterMode::Position => {
                let pos = usize::try_from(pos).map_err(|_| ComputerError::ReadOutsideOfMemory)?;
                Ok(self.memory.get(pos))
            }
            ParameterMode::Relative => {
                let pos = usize::try_from(self.relative_base + pos)
                    .map_err(|_| ComputerError::ReadOutsideOfMemory)?;
                Ok(self.memory.get(pos))
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
                Ok(self.memory.get_mut(pos))
            }
            ParameterMode::Relative => {
                let pos = usize::try_from(self.relative_base + pos)
                    .map_err(|_| ComputerError::ReadOutsideOfMemory)?;
                Ok(self.memory.get_mut(pos))
            }
        }
    }

    pub fn base_memory(&self) -> &[Value] {
        &self.memory.base
    }
}
