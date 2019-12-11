#![feature(proc_macro_hygiene, stmt_expr_attributes)]
#![feature(async_closure)]

pub mod day1;
pub mod day2;
pub mod day3;
pub mod day4;
pub mod day5;
pub mod day_10;
pub mod day_6;
pub mod day_7;
pub mod day_8;
pub mod day_9;
pub mod intcode;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static>>;
