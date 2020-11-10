use anyhow::{anyhow, Error};

use std::collections::HashMap;
use std::convert::Into;
use std::ops::Deref;

use super::path::{Path, Step};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Movement {
    Forward(usize),
    TurnLeft,
    TurnRight,
}

impl ToString for Movement {
    fn to_string(&self) -> String {
        match self {
            Movement::Forward(steps) => format!("{}", steps),
            Movement::TurnLeft => "L".to_string(),
            Movement::TurnRight => "R".to_string(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MovementProgram(Vec<Movement>);

impl From<Path> for MovementProgram {
    fn from(path: Path) -> Self {
        let mut cmds = Vec::new();

        let mut forward = 0;

        for step in path.steps() {
            match step {
                Step::Forward => {
                    forward += 1;
                }
                Step::Left => {
                    if forward > 0 {
                        cmds.push(Movement::Forward(forward));
                    }
                    forward = 0;
                    cmds.push(Movement::TurnLeft);
                }
                Step::Right => {
                    if forward > 0 {
                        cmds.push(Movement::Forward(forward));
                    }
                    forward = 0;
                    cmds.push(Movement::TurnRight);
                }
            }
        }

        if forward > 0 {
            cmds.push(Movement::Forward(forward));
        }

        MovementProgram(cmds)
    }
}

impl From<Vec<Movement>> for MovementProgram {
    fn from(v: Vec<Movement>) -> Self {
        MovementProgram(v)
    }
}

impl ToString for MovementProgram {
    fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<String>>()
            .join(",")
    }
}

impl MovementProgram {
    fn starts_with(&self, other: &[Movement]) -> bool {
        if other.len() > self.len() {
            return false;
        }

        self.0.starts_with(other)
    }

    fn size(&self) -> usize {
        self.to_string().len()
    }

    fn is_small(&self) -> bool {
        self.size() <= 20
    }
}

impl Deref for MovementProgram {
    type Target = [Movement];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
enum Routine {
    A,
    B,
    C,
}

const ROUTINES: [Routine; 3] = [Routine::A, Routine::B, Routine::C];

impl From<usize> for Routine {
    fn from(idx: usize) -> Self {
        match idx {
            0 => Routine::A,
            1 => Routine::B,
            2 => Routine::C,
            _ => panic!("Unexpected routine index"),
        }
    }
}

impl ToString for Routine {
    fn to_string(&self) -> String {
        match self {
            Routine::A => "A".to_string(),
            Routine::B => "B".to_string(),
            Routine::C => "C".to_string(),
        }
    }
}

#[derive(Debug, Default)]
struct MovementRoutine(Vec<Routine>);

impl ToString for MovementRoutine {
    fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<String>>()
            .join(",")
    }
}

#[derive(Debug, Default)]
pub struct MovementPrograms {
    main: MovementRoutine,
    routines: HashMap<Routine, MovementProgram>,
}

impl ToString for MovementPrograms {
    fn to_string(&self) -> String {
        use std::fmt::Write;

        let mut buf = String::new();

        writeln!(buf, "{}", self.main.to_string()).unwrap();

        for routine in &ROUTINES {
            writeln!(
                buf,
                "{}",
                self.routines
                    .get(routine)
                    .map(|m| m.to_string())
                    .unwrap_or("".to_string())
            )
            .unwrap();
        }

        buf
    }
}

impl MovementPrograms {
    pub fn compile<P>(program: P) -> Result<MovementPrograms, Error>
    where
        P: Into<MovementProgram>,
    {
        let p = program.into();
        for a in 1..=11 {
            for b in 1..=11 {
                for c in 1..=11 {
                    match MovementPrograms::build(&p, a, b, c) {
                        Ok(r) => {
                            return Ok(r);
                        }
                        Err(_) => {}
                    }
                }
            }
        }
        Err(anyhow!("Unable to build program!"))
    }

    fn build(
        program: &MovementProgram,
        a: usize,
        b: usize,
        c: usize,
    ) -> Result<MovementPrograms, Error> {
        let r_a: MovementProgram = program.iter().take(a).copied().collect::<Vec<_>>().into();

        if !r_a.is_small() {
            return Err(anyhow!("Routine A does not fit in memory: {:?}", r_a));
        }

        let mut programs = MovementPrograms::default();
        programs.routines.insert(Routine::A, r_a);

        let mut remainder = program.clone();

        loop {
            match programs.strip(&remainder) {
                Some((r, p)) => {
                    programs.main.0.push(r);
                    remainder = p;
                }
                None => {
                    if remainder.is_empty() {
                        return Ok(programs);
                    }
                    if !programs.routines.contains_key(&Routine::B) {
                        let r_b: MovementProgram =
                            remainder.iter().take(b).copied().collect::<Vec<_>>().into();
                        if !r_b.is_small() {
                            return Err(anyhow!("Routine B does not fit in memory: {:?}", r_b));
                        }

                        programs.routines.insert(Routine::B, r_b.into());
                    } else if !programs.routines.contains_key(&Routine::C) {
                        let r_c: MovementProgram =
                            remainder.iter().take(c).copied().collect::<Vec<_>>().into();
                        if !r_c.is_small() {
                            return Err(anyhow!("Routine B does not fit in memory: {:?}", r_c));
                        }
                        programs.routines.insert(Routine::C, r_c);
                    } else {
                        return Err(anyhow!(
                            "Unable to consume program: {:?} {:?}",
                            program,
                            programs
                        ));
                    }
                }
            }

            if programs.main.0.len() >= 11 && !remainder.is_empty() {
                return Err(anyhow!(
                    "Insufficient memory in main routine: {:?} {:?}",
                    programs,
                    remainder
                ));
            }
        }
    }

    fn strip(&self, program: &MovementProgram) -> Option<(Routine, MovementProgram)> {
        for (i, subprogram) in self.routines.iter() {
            if program.starts_with(subprogram) {
                let remiander: MovementProgram = program
                    .iter()
                    .skip(subprogram.len())
                    .copied()
                    .collect::<Vec<_>>()
                    .into();

                return Some((*i, remiander));
            }
        }
        None
    }

    pub fn expand(&self) -> MovementProgram {
        let mut result = Vec::new();

        for routine in &self.main.0 {
            result.extend(self.routines.get(routine).unwrap().iter())
        }

        result.into()
    }
}
