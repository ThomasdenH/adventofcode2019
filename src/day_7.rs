use crate::intcode::{Computer, ComputerError, Value};
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

async fn output(memory: Vec<Value>, mut input: &[Value]) -> Result<Value, ComputerError> {
    let mut output_a = &mut None;
    Computer::load(memory)
        .run(&mut input, Some(&mut output_a))
        .await?;
    Ok(output_a.unwrap())
}

/// Parse the input to a common format between both parts.
pub fn parse_input(input: &str) -> Result<Vec<Value>, SolutionError> {
    input
        .trim()
        .split(',')
        .map(|s| s.parse::<Value>().map_err(SolutionError::from))
        .collect()
}

/// Solve the first part for the parsed input.
pub async fn part_1(parsed_input: &[Value]) -> Result<Value, SolutionError> {
    let mut max = None;
    for perm in Heap::new(&mut [0, 1, 2, 3, 4]) {
        let mut previous = 0;
        for phase_setting in &perm {
            previous = output(parsed_input.to_vec(), &[*phase_setting, previous]).await?;
        }
        max = Some(max.map(|m: Value| m.max(previous)).unwrap_or(previous));
    }
    Ok(max.unwrap())
}

const BUFFER_SIZE: usize = 1;
const LAST_AMPLIFIER: usize = 4;

/// Solve the second part for the parsed input.
pub async fn part_2(parsed_input: &[Value]) -> Result<Value, SolutionError> {
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

        dbg!("senders and receivers ready");

        let mut receivers = futures::future::try_join_all(perm.iter().zip(senders.into_iter()).zip(receivers.into_iter()).enumerate().map(
            async move |(i, ((phase_setting, mut sender), mut receiver))| -> Result<Receiver<Value>, SolutionError> {
                dbg!("start");
                // Send the phase signal to the next amplifier, because order doesn't matter
                sender.send(*phase_setting).await?;
                if i == LAST_AMPLIFIER {
                    // Send the 0 signal to the first amplifier, via the sender of the last one
                    sender.send(0).await?;
                }
                dbg!("sent signals");
                Computer::load(parsed_input.to_vec())
                    .run(&mut receiver, Some(&mut sender)).await?;
                dbg!("done running");
                sender.close().await?;
                Ok(receiver)
            },
        ))
        .await?;
        dbg!("gotten receivers");
        let output = receivers.remove(0).next().await.unwrap();
        max = Some(max.map(|m| m.max(output)).unwrap_or(output));
    }
    Ok(max.unwrap())
}
