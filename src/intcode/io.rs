use async_trait::async_trait;
use futures::channel::mpsc::{Receiver, Sender};
use futures::prelude::*;
use futures_await_test::async_test;

use crate::intcode::Value;

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
impl Write for Option<Value> {
    async fn write(&mut self, output: Value) {
        self.replace(output);
    }
}

#[async_trait]
impl Write for Sender<Value> {
    async fn write(&mut self, output: Value) {
        self.send(output).await.unwrap();
    }
}
