use crate::intcode::{parse_program, Computer, ComputerError, Memory, Value};
use futures::channel::mpsc::{channel, Receiver, SendError, Sender};
use futures::prelude::*;
use permutohedron::Heap;
use std::fmt::Debug;
use thiserror::*;

#[derive(Error, Debug)]
pub enum SolutionError {
    #[error("could not parse the input ints")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("a computer error occurred")]
    ComputerError(#[from] ComputerError),
    #[error("could not send input")]
    SendError(#[from] SendError),
}

pub struct Solution {}

async fn output(memory: Memory, mut input: &[Value]) -> Result<Value, ComputerError> {
    let mut output_a = None;
    let mut comp = Computer::load(memory);
    comp.set_input(Some(&mut input));
    comp.set_output(Some(&mut output_a));
    comp.run().await?;
    Ok(output_a.unwrap())
}

/// Parse the input to a common format between both parts.
pub fn parse_input(input: &str) -> Result<Memory, SolutionError> {
    parse_program(input).map_err(|e| e.into())
}

/// Solve the first part for the parsed input.
pub async fn part_1(parsed_input: Memory) -> Result<Value, SolutionError> {
    let mut max = None;
    for perm in Heap::new(&mut [0, 1, 2, 3, 4]) {
        let mut previous = 0;
        for phase_setting in &perm {
            previous = output(parsed_input.clone(), &[*phase_setting, previous]).await?;
        }
        max = Some(max.map(|m: Value| m.max(previous)).unwrap_or(previous));
    }
    Ok(max.unwrap())
}

const BUFFER_SIZE: usize = 1;
const LAST_AMPLIFIER: usize = 4;

/// Solve the second part for the parsed input.
pub async fn part_2(parsed_input: &Memory) -> Result<Value, SolutionError> {
    let mut max: Option<Value> = None;
    let phase_settings: &mut [Value] = &mut [5, 6, 7, 8, 9];
    for perm in Heap::new(phase_settings) {
        let (mut senders, receivers): (Vec<Sender<Value>>, Vec<Receiver<Value>>) =
            perm.iter().map(|_| channel(BUFFER_SIZE)).fold(
                (Vec::new(), Vec::new()),
                |(mut send_vec, mut rec_vec), (sender, receiver)| {
                    send_vec.push(sender);
                    rec_vec.push(receiver);
                    (send_vec, rec_vec)
                },
            );
        // Change order of senders
        let first = senders.remove(0);
        senders.push(first);

        let mut receivers = futures::future::try_join_all(perm.iter().zip(senders.into_iter()).zip(receivers.into_iter()).enumerate().map(
            async move |(i, ((phase_setting, mut sender), mut receiver))| -> Result<Receiver<Value>, SolutionError> {
                // Send the phase signal to the next amplifier, because order doesn't matter
                sender.send(*phase_setting).await?;
                if i == LAST_AMPLIFIER {
                    // Send the 0 signal to the first amplifier, via the sender of the last one
                    sender.send(0).await?;
                }
                let mut comp = Computer::load(parsed_input.clone());
                comp.set_input(Some(&mut receiver));
                comp.set_output(Some(&mut sender));
                comp.run().await?;
                sender.close().await?;
                Ok(receiver)
            },
        ))
        .await?;
        let output = receivers.remove(0).next().await.unwrap();
        max = Some(max.map(|m| m.max(output)).unwrap_or(output));
    }
    Ok(max.unwrap())
}
