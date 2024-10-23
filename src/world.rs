//! Câmpul de lucru: o grilă de noduri pe care se mișcă cangurul.

use std::collections::HashSet;

use crate::lang::{Action, Cond, CondKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Dir {
    East,
    South,
    West,
    North,
}

impl Dir {
    /// Rotire cu 90° în sensul acelor ceasornicului.
    pub fn clockwise(self) -> Dir {
        match self {
            Dir::East => Dir::South,
            Dir::South => Dir::West,
            Dir::West => Dir::North,
            Dir::North => Dir::East,
        }
    }

    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::East => (1, 0),
            Dir::South => (0, 1),
            Dir::West => (-1, 0),
            Dir::North => (0, -1),
        }
    }
}

/// O mișcare executată (pentru animație și istoric).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    pub action: Action,
    pub from: (i32, i32),
    pub to: (i32, i32),
    pub dir_before: Dir,
    pub dir_after: Dir,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldError {
    /// Cangurul ar ieși din câmp.
    OutOfField,
}

#[derive(Clone, Debug)]
pub struct World {
    /// Numărul de celule pe orizontală / verticală (nodurile sunt 0..=width).
    pub width: i32,
    pub height: i32,
    pub pos: (i32, i32),
    pub dir: Dir,
    /// Segmentele desenate, în ordinea desenării (capete ordonate).
    pub lines: Vec<((i32, i32), (i32, i32))>,
    line_set: HashSet<((i32, i32), (i32, i32))>,
    /// Noduri prin care trece cel puțin o linie.
    marked: HashSet<(i32, i32)>,
    /// Salturile, pentru a putea fi arătate opțional.
    pub jumps: Vec<((i32, i32), (i32, i32))>,
    pub step_count: usize,
}

impl World {
    pub fn new(width: i32, height: i32) -> World {
        World {
            width,
            height,
            pos: (0, 0),
            dir: Dir::East,
            lines: Vec::new(),
            line_set: HashSet::new(),
            marked: HashSet::new(),
            jumps: Vec::new(),
            step_count: 0,
        }
    }

    pub fn inside(&self, (x, y): (i32, i32)) -> bool {
        (0..=self.width).contains(&x) && (0..=self.height).contains(&y)
    }

    pub fn ahead(&self) -> (i32, i32) {
        let (dx, dy) = self.dir.delta();
        (self.pos.0 + dx, self.pos.1 + dy)
    }

    pub fn check(&self, c: Cond) -> bool {
        let ahead = self.ahead();
        let v = match c.kind {
            CondKind::Border => !self.inside(ahead),
            // Marginea câmpului este considerată și ea o linie.
            CondKind::Line => !self.inside(ahead) || self.marked.contains(&ahead),
        };
        v != c.negated
    }

    pub fn apply(&mut self, a: Action) -> Result<Move, WorldError> {
        let from = self.pos;
        let dir_before = self.dir;
        match a {
            Action::Rotate => self.dir = self.dir.clockwise(),
            Action::Step | Action::Jump => {
                let to = self.ahead();
                if !self.inside(to) {
                    return Err(WorldError::OutOfField);
                }
                self.pos = to;
                if a == Action::Step {
                    let key = if from <= to { (from, to) } else { (to, from) };
                    if self.line_set.insert(key) {
                        self.lines.push(key);
                    }
                    self.marked.insert(from);
                    self.marked.insert(to);
                } else {
                    self.jumps.push((from, to));
                }
            }
        }
        self.step_count += 1;
        Ok(Move {
            action: a,
            from,
            to: self.pos,
            dir_before,
            dir_after: self.dir,
        })
    }
}
