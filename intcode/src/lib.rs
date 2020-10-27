mod cpu;
mod errors;
mod opcode;
mod program;

pub use crate::cpu::{CPUState, Computer};
pub use crate::errors::{IntcodeError, Result};
pub use crate::opcode::OpCode;
pub use crate::program::{Arguments, Assembly, Program};

pub type IntMem = i64;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn example_program_day_2() {
        let program = vec![1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50];

        let mut cpu = Computer::new(program);
        cpu.run().unwrap();
        let final_state = cpu.program();
        assert_eq!(final_state.get(0), Some(3500));
        assert_eq!(final_state.get(3), Some(70))
    }

    fn transform(program: Vec<IntMem>) -> Result<Vec<IntMem>> {
        let mut cpu = Computer::new(program);
        cpu.run()?;

        Ok(cpu.program().tape())
    }

    #[test]
    fn example_cases_day_2() {
        assert_eq!(
            transform(vec![1, 0, 0, 0, 99]).unwrap(),
            vec![2, 0, 0, 0, 99]
        );
        assert_eq!(
            transform(vec![2, 3, 0, 3, 99]).unwrap(),
            vec![2, 3, 0, 6, 99]
        );
        assert_eq!(
            transform(vec![2, 4, 4, 5, 99, 0]).unwrap(),
            vec![2, 4, 4, 5, 99, 9801]
        );
        assert_eq!(
            transform(vec![1, 1, 1, 4, 99, 5, 6, 0, 99]).unwrap(),
            vec![30, 1, 1, 4, 2, 5, 6, 0, 99]
        );
    }

    #[test]
    fn example_case_day_5() {
        let mut cpu = Computer::new(vec![3, 0, 4, 0, 99]);
        assert_eq!(cpu.simple(5).unwrap(), 5);
    }

    #[test]
    fn example_case_day9() {
        let program = vec![
            109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
        ];
        let mut cpu = Computer::new(program.clone());
        assert_eq!(cpu.follow().collect::<Vec<IntMem>>(), program);
    }

    #[test]
    fn example_case_day9_bignum() {
        let program = vec![1102, 34915192, 34915192, 7, 4, 7, 99, 0];

        let mut cpu = Computer::new(program.clone());
        let value = cpu.follow().one().expect("Single Output");
        assert_eq!(format!("{}", value).len(), 16)
    }

    #[test]
    fn example_case_day9_bignum_simple() {
        let program = vec![104, 1125899906842624, 99];

        let mut cpu = Computer::new(program.clone());
        let value = cpu.follow().one().unwrap();

        assert_eq!(value, 1125899906842624)
    }
}
