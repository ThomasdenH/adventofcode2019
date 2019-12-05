use crate::Result;

const OP_QUIT: usize = 99;
const OP_ADD: usize = 1;
const OP_MUL: usize = 2;

pub fn parse_input(s: &str) -> Result<Vec<usize>> {
    s.trim()
        .split(',')
        .map(|s| s.parse::<usize>().map_err(|e| e.into()))
        .collect()
}

pub fn run_program(prog: &mut [usize]) -> &[usize] {
    let mut program_counter = 0usize;
    loop {
        if program_counter >= prog.len() {
            return prog;
        }
        match prog[program_counter] {
            OP_QUIT => return prog,
            OP_ADD => {
                let a_at = prog[program_counter + 1];
                let b_at = prog[program_counter + 2];
                let result_to = prog[program_counter + 3];
                prog[result_to] = prog[a_at] + prog[b_at];
                program_counter += 4;
            }
            OP_MUL => {
                let a_at = prog[program_counter + 1];
                let b_at = prog[program_counter + 2];
                let result_to = prog[program_counter + 3];
                prog[result_to] = prog[a_at] * prog[b_at];
                program_counter += 4;
            }
            other => panic!("unknown opcode {}", other),
        }
    }
}

#[test]
fn test_examples() {
    assert_eq!(run_program(&mut [1, 0, 0, 0, 99]), &[2, 0, 0, 0, 99]);
}

pub fn part_1(input: &mut [usize]) -> usize {
    input[1] = 12;
    input[2] = 2;
    run_program(input)[0]
}

pub fn part_2(input: Vec<usize>) -> usize {
    for i in 0..100 {
        for j in 0..100 {
            let mut input = input.clone();
            input[1] = i;
            input[2] = j;
            if run_program(&mut input)[0] == 19_690_720 {
                return 100 * i + j;
            }
        }
    }
    unreachable!();
}
