//! Mașina virtuală care execută un `Program` câte o acțiune pe rând,
//! pentru a putea anima cangurul și a evidenția linia curentă.

use crate::lang::{Instr, Program};
use crate::world::{Move, World, WorldError};

/// Adâncimea maximă a apelurilor de proceduri („Stack-ul este umplut”).
pub const MAX_CALL_DEPTH: usize = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeError {
    OutOfField,
    StackOverflow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tick {
    /// S-a executat o acțiune pe linia dată.
    Moved(Move, usize),
    /// Bugetul de instrucțiuni s-a terminat fără nicio acțiune (ex. ciclu gol).
    Busy,
    Finished,
    Failed(RuntimeError, usize),
}

pub struct Vm {
    program: Program,
    pc: usize,
    calls: Vec<usize>,
    counts: Vec<u32>,
    done: bool,
}

impl Vm {
    pub fn new(program: Program) -> Vm {
        let pc = program.entry;
        Vm {
            program,
            pc,
            calls: Vec::new(),
            counts: Vec::new(),
            done: false,
        }
    }

    /// Execută instrucțiuni până la prima acțiune (sau până se termină bugetul).
    pub fn tick(&mut self, world: &mut World, budget: usize) -> Tick {
        if self.done {
            return Tick::Finished;
        }
        for _ in 0..budget {
            let line = self.program.lines[self.pc];
            match self.program.code[self.pc].clone() {
                Instr::Act(a) => match world.apply(a) {
                    Ok(m) => {
                        self.pc += 1;
                        return Tick::Moved(m, line);
                    }
                    Err(WorldError::OutOfField) => {
                        return self.fail(RuntimeError::OutOfField, line);
                    }
                },
                Instr::PushCount(n) => {
                    self.counts.push(n);
                    self.pc += 1;
                }
                Instr::CountDown(end) => {
                    let top = self.counts.last_mut().expect("contor lipsă");
                    if *top == 0 {
                        self.counts.pop();
                        self.pc = end;
                    } else {
                        *top -= 1;
                        self.pc += 1;
                    }
                }
                Instr::Jump(t) => self.pc = t,
                Instr::JumpUnless(c, t) => {
                    self.pc = if world.check(c) { self.pc + 1 } else { t };
                }
                Instr::Call(t) => {
                    if self.calls.len() >= MAX_CALL_DEPTH {
                        return self.fail(RuntimeError::StackOverflow, line);
                    }
                    self.calls.push(self.pc + 1);
                    self.pc = t;
                }
                Instr::Return => self.pc = self.calls.pop().expect("return fără apel"),
                Instr::Halt => {
                    self.done = true;
                    return Tick::Finished;
                }
            }
        }
        Tick::Busy
    }

    fn fail(&mut self, e: RuntimeError, line: usize) -> Tick {
        self.done = true;
        Tick::Failed(e, line)
    }
}

/// Rulează un program până la capăt (folosit la teste).
#[cfg(test)]
pub fn run_to_end(vm: &mut Vm, world: &mut World, max_actions: usize) -> Tick {
    let mut actions = 0;
    loop {
        match vm.tick(world, 100_000) {
            Tick::Moved(..) => {
                actions += 1;
                if actions >= max_actions {
                    return Tick::Busy;
                }
            }
            Tick::Busy => return Tick::Busy,
            t => return t,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::compile;

    fn run(src: &str, w: i32, h: i32) -> (World, Tick) {
        let mut vm = Vm::new(compile(src).unwrap());
        let mut world = World::new(w, h);
        let t = run_to_end(&mut vm, &mut world, 1_000_000);
        (world, t)
    }

    fn example(name: &str) -> String {
        std::fs::read_to_string(format!("{}/examples/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap()
    }

    #[test]
    fn square() {
        let (w, t) = run("[ repeta 4 ori pas rotire sfirsitul repetarii ]", 5, 5);
        assert_eq!(t, Tick::Finished);
        assert_eq!(w.lines.len(), 4);
        assert_eq!(w.pos, (0, 0));
    }

    #[test]
    fn out_of_field() {
        let (_, t) = run("[\nrotire rotire\npas\n]", 5, 5);
        assert!(matches!(t, Tick::Failed(RuntimeError::OutOfField, 3)));
    }

    #[test]
    fn recursion_overflows() {
        let (_, t) = run(
            "procedura a\nexecuta a\nsfirsitul procedurii\n[ executa a ]",
            5,
            5,
        );
        assert!(matches!(t, Tick::Failed(RuntimeError::StackOverflow, _)));
    }

    #[test]
    fn spiral_in_all_languages() {
        for f in ["Spirala(Rom).cng", "Spirala(Eng).cng", "Spirala(Rus).cng"] {
            let (w, t) = run(&example(f), 10, 10);
            assert_eq!(t, Tick::Finished, "{f}");
            assert!(w.lines.len() > 30, "{f}: {}", w.lines.len());
        }
    }

    #[test]
    fn original_examples_run() {
        for f in [
            "Cangur.cng",
            "figura1.cng",
            "figura2.cng",
            "figura3.cng",
            "Litera_L.cng",
        ] {
            let (w, t) = run(&example(f), 20, 20);
            assert_eq!(t, Tick::Finished, "{f}");
            assert!(!w.lines.is_empty(), "{f}");
        }
    }

    #[test]
    fn if_else_parses() {
        assert!(compile(&example("Patrat.cng")).is_ok());
    }
}
