//! Analiza programului înainte de rulare: proceduri necunoscute, avertismente
//! despre cicluri care pot rula la nesfârșit și sugestii pentru cuvinte greșite.

use std::collections::{HashMap, HashSet};
use std::ops::Range;

use crate::lang::{Action, Ast, CompileError, ErrorKind, Stmt, StmtKind, normalize};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WarningKind {
    UnusedProcedure(String),
    EmptyBlock,
    RepeatZero,
    /// CÎT fără nicio mișcare în corp: poate rula la nesfârșit.
    LoopWithoutMovement,
    /// Procedura se apelează pe ea însăși fără nicio condiție.
    EndlessRecursion(String),
    EmptyProgram,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Warning {
    pub line: usize,
    pub span: Range<usize>,
    pub kind: WarningKind,
}

/// Cuvinte pe care elevii le scriu des în loc de comenzi.
const SYNONYMS: &[(&str, &str)] = &[
    ("SARI", "SALT"),
    ("SARE", "SALT"),
    ("SARITURA", "SALT"),
    ("ROTESTE", "ROTIRE"),
    ("INTOARCE", "ROTIRE"),
    ("MERGI", "PAS"),
    ("MERGE", "PAS"),
    ("PASESTE", "PAS"),
    ("REPETAM", "REPETA"),
    ("SFIRSIT", "SFIRSITUL"),
    ("SFARSIT", "SFIRSITUL"),
];

fn distance(a: &[char], b: &[char]) -> usize {
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            cur.push(
                (prev[j] + usize::from(ca != cb))
                    .min(prev[j + 1] + 1)
                    .min(cur[j] + 1),
            );
        }
        prev = cur;
    }
    prev[b.len()]
}

/// Scrie sugestia cu aceleași litere (mici / mari) ca originalul.
fn match_case(original: &str, word: &str) -> String {
    let mut chars = original.chars().filter(|c| c.is_alphabetic());
    let first_upper = chars.next().is_some_and(char::is_uppercase);
    let rest_lower = chars.all(|c| !c.is_uppercase());
    match (first_upper, rest_lower) {
        (false, _) => word.to_lowercase(),
        (true, true) => {
            let lower = word.to_lowercase();
            let mut c = lower.chars();
            c.next()
                .map_or(String::new(), |f| f.to_uppercase().chain(c).collect())
        }
        (true, false) => word.to_string(),
    }
}

/// Cel mai apropiat cuvânt din listă de un cuvânt scris greșit.
pub fn suggest(normalized: &str, original: &str, candidates: &[&str]) -> Option<String> {
    if let Some((_, to)) = SYNONYMS.iter().find(|(from, _)| *from == normalized)
        && candidates.contains(to)
    {
        return Some(match_case(original, to));
    }
    let w: Vec<char> = normalized.chars().collect();
    let max = match w.len() {
        0..=4 => 1,
        5..=8 => 2,
        _ => 3,
    };
    candidates
        .iter()
        .map(|c| (distance(&w, &c.chars().collect::<Vec<_>>()), *c))
        .filter(|&(d, _)| d > 0 && d <= max)
        .min_by_key(|&(d, _)| d)
        .map(|(_, c)| match_case(original, c))
}

/// Comenzile din corp mută cangurul (sau apelează o procedură)?
fn moves(body: &[Stmt]) -> bool {
    body.iter().any(|s| match &s.kind {
        StmtKind::Act(a) => *a != Action::Rotate,
        StmtKind::Call(..) => true,
        StmtKind::Repeat { body, .. } | StmtKind::While(_, body) => moves(body),
        StmtKind::If(_, a, b) => {
            moves(std::slice::from_ref(a))
                || b.as_deref().is_some_and(|b| moves(std::slice::from_ref(b)))
        }
        StmtKind::Invalid => true,
    })
}

/// Apelurile care se fac sigur, fără nicio condiție (pentru recursivitate infinită).
fn unconditional_calls<'a>(body: &'a [Stmt], out: &mut Vec<&'a str>) {
    for s in body {
        match &s.kind {
            StmtKind::Call(name, ..) => out.push(name),
            StmtKind::Repeat { n, body, .. } if *n > 0 => unconditional_calls(body, out),
            _ => {}
        }
    }
}

struct Checker<'a> {
    procs: HashMap<&'a str, usize>,
    proc_texts: Vec<&'a str>,
    errors: &'a mut Vec<CompileError>,
    warnings: Vec<Warning>,
    calls: Vec<HashSet<usize>>,
}

impl<'a> Checker<'a> {
    fn warn(&mut self, line: usize, span: &Range<usize>, kind: WarningKind) {
        self.warnings.push(Warning {
            line,
            span: span.clone(),
            kind,
        });
    }

    /// Verifică un corp; `owner` este procedura care îl conține (`None` = programul principal).
    fn body(&mut self, body: &'a [Stmt], owner: usize) {
        for s in body {
            match &s.kind {
                StmtKind::Call(name, text, span) => match self.procs.get(name.as_str()) {
                    Some(&i) => {
                        self.calls[owner].insert(i);
                    }
                    None => {
                        let names: Vec<String> =
                            self.proc_texts.iter().map(|t| normalize(t)).collect();
                        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
                        let hint = suggest(name, text, &refs)
                            .and_then(|h| {
                                self.proc_texts
                                    .iter()
                                    .find(|t| normalize(t) == normalize(&h))
                            })
                            .map(|t| t.to_string());
                        self.errors.push(CompileError {
                            line: s.line,
                            span: span.clone(),
                            kind: ErrorKind::UnknownProcedure(text.clone()),
                            hint,
                        });
                    }
                },
                StmtKind::Repeat { n, n_span, body } => {
                    if *n == 0 {
                        self.warn(s.line, n_span, WarningKind::RepeatZero);
                    } else if body.is_empty() {
                        self.warn(s.line, &s.span, WarningKind::EmptyBlock);
                    }
                    self.body(body, owner);
                }
                StmtKind::While(_, body) => {
                    if !moves(body) {
                        self.warn(s.line, &s.span, WarningKind::LoopWithoutMovement);
                    }
                    self.body(body, owner);
                }
                StmtKind::If(_, a, b) => {
                    self.body(std::slice::from_ref(a), owner);
                    if let Some(b) = b {
                        self.body(std::slice::from_ref(b), owner);
                    }
                }
                StmtKind::Act(_) | StmtKind::Invalid => {}
            }
        }
    }
}

/// Analiza semantică; adaugă erorile în `errors` și întoarce avertismentele.
pub(crate) fn check(ast: &Ast, errors: &mut Vec<CompileError>) -> Vec<Warning> {
    let n = ast.procs.len();
    let main = n; // indexul „programului principal” în graful de apeluri
    let mut c = Checker {
        procs: ast
            .procs
            .iter()
            .enumerate()
            .map(|(i, p)| (p.name.as_str(), i))
            .collect(),
        proc_texts: ast.procs.iter().map(|p| p.text.as_str()).collect(),
        errors,
        warnings: Vec::new(),
        calls: vec![HashSet::new(); n + 1],
    };

    if ast.main.is_empty() {
        let (line, span) = ast.main_at.clone().unwrap_or((1, 0..0));
        c.warn(line, &span, WarningKind::EmptyProgram);
    }
    c.body(&ast.main, main);
    for (i, p) in ast.procs.iter().enumerate() {
        if p.body.is_empty() {
            c.warn(p.line, &p.span, WarningKind::EmptyBlock);
        }
        c.body(&p.body, i);
    }

    // Procedurile la care nu se ajunge din programul principal.
    let mut used = HashSet::new();
    let mut stack = vec![main];
    while let Some(i) = stack.pop() {
        for &j in &c.calls[i] {
            if used.insert(j) {
                stack.push(j);
            }
        }
    }
    for (i, p) in ast.procs.iter().enumerate() {
        if !used.contains(&i) {
            c.warn(
                p.line,
                &p.span,
                WarningKind::UnusedProcedure(p.text.clone()),
            );
        }
    }

    // Recursivitate fără oprire: un ciclu din apeluri făcute fără condiție.
    let sure: Vec<Vec<usize>> = ast
        .procs
        .iter()
        .map(|p| {
            let mut names = Vec::new();
            unconditional_calls(&p.body, &mut names);
            names
                .iter()
                .filter_map(|n| c.procs.get(n).copied())
                .collect()
        })
        .collect();
    for (i, p) in ast.procs.iter().enumerate() {
        let mut seen = HashSet::new();
        let mut stack = sure[i].clone();
        let mut cycle = false;
        while let Some(j) = stack.pop() {
            if j == i {
                cycle = true;
                break;
            }
            if seen.insert(j) {
                stack.extend(&sure[j]);
            }
        }
        if cycle {
            c.warn(
                p.line,
                &p.span,
                WarningKind::EndlessRecursion(p.text.clone()),
            );
        }
    }

    let mut warnings = c.warnings;
    warnings.sort_by_key(|w| (w.span.start, w.line));
    warnings
}

#[cfg(test)]
mod tests {
    use crate::lang::{BlockKind, ErrorKind, analyze};

    use super::*;

    fn kinds(src: &str) -> Vec<(usize, ErrorKind)> {
        analyze(src)
            .errors
            .into_iter()
            .map(|e| (e.line, e.kind))
            .collect()
    }

    #[test]
    fn reports_all_errors_not_just_first() {
        let src = "[\nPAS\nSARI\nREPETĂ de 3 ORI\n    PAS\nSFÎRȘITUL REPETĂRII\nCÎT NU E_MARGNE\n    PAS\nSFÎRȘITUL CICLULUI\nEXECUTĂ Patrat\n]\n";
        assert_eq!(
            kinds(src),
            vec![
                (3, ErrorKind::UnknownWord("SARI".into())),
                (4, ErrorKind::ExpectedNumber),
                (7, ErrorKind::UnknownCondition("E_MARGNE".into())),
                (10, ErrorKind::UnknownProcedure("Patrat".into())),
            ]
        );
        let a = analyze(src);
        assert_eq!(a.errors[0].hint.as_deref(), Some("SALT"));
        assert_eq!(a.errors[2].hint.as_deref(), Some("E_MARGINE"));
        assert_eq!(&src[a.errors[0].span.clone()], "SARI");
        assert!(a.program.is_none());
    }

    #[test]
    fn hints_keep_case() {
        assert_eq!(
            suggest("ROTRE", "rotre", crate::lang::COMMAND_WORDS).as_deref(),
            Some("rotire")
        );
        assert_eq!(
            suggest("REPTA", "Repta", crate::lang::COMMAND_WORDS).as_deref(),
            Some("Repeta")
        );
        assert_eq!(
            suggest("ZZZZZZ", "zzzzzz", crate::lang::COMMAND_WORDS),
            None
        );
    }

    #[test]
    fn block_structure_errors() {
        // Lipsește sfârșitul repetării: eroarea e pe rândul cu REPETĂ.
        assert_eq!(
            kinds("[\nREPETĂ 2 ORI\nPAS\n]"),
            vec![(2, ErrorKind::MissingEnd(BlockKind::Repeat))]
        );
        assert_eq!(
            kinds("[\nPAS\nSFÎRȘITUL CICLULUI\n]"),
            vec![(3, ErrorKind::StrayEnd)]
        );
        assert_eq!(
            kinds("[\nDACĂ E_MARGINE PAS PAS ALTFEL ROTIRE\n]"),
            vec![(2, ErrorKind::StrayElse)]
        );
        assert_eq!(kinds("[\nPAS\n"), vec![(1, ErrorKind::MissingMainEnd)]);
        assert_eq!(
            kinds("[\nPAS\n]\nPAS"),
            vec![(4, ErrorKind::TextAfterProgram)]
        );
        assert_eq!(
            kinds("[\nPAS\nPROCEDURA A\nPAS\nSFÎRȘITUL PROCEDURII\n]"),
            vec![(3, ErrorKind::ProcedureInMain)]
        );
        assert_eq!(
            kinds("PROCEDURA A\nPAS\n[\nEXECUTĂ A\n]"),
            vec![(1, ErrorKind::MissingEnd(BlockKind::Procedure))]
        );
        assert_eq!(
            kinds("[\nDACĂ E_MARGINE\n]"),
            vec![(3, ErrorKind::ExpectedCommand)]
        );
    }

    #[test]
    fn warnings() {
        let a = analyze(
            "PROCEDURA Nefolosita\nPAS\nSFÎRȘITUL PROCEDURII\n\
             PROCEDURA R\nPAS\nEXECUTĂ R\nSFÎRȘITUL PROCEDURII\n\
             [\nREPETĂ 0 ORI\nPAS\nSFÎRȘITUL REPETĂRII\nCÎT NU E_MARGINE\nROTIRE\nSFÎRȘITUL CICLULUI\nEXECUTĂ R\n]",
        );
        assert!(a.errors.is_empty(), "{:?}", a.errors);
        assert!(a.program.is_some(), "avertismentele nu opresc rularea");
        let w: Vec<_> = a.warnings.into_iter().map(|w| (w.line, w.kind)).collect();
        assert_eq!(
            w,
            vec![
                (1, WarningKind::UnusedProcedure("Nefolosita".into())),
                (4, WarningKind::EndlessRecursion("R".into())),
                (9, WarningKind::RepeatZero),
                (12, WarningKind::LoopWithoutMovement),
            ]
        );
    }

    #[test]
    fn original_examples_are_clean() {
        for f in [
            "Cangur.cng",
            "Litera_L.cng",
            "Spirala(Rom).cng",
            "Spirala(Rus).cng",
            "figura1.cng",
        ] {
            let src =
                std::fs::read_to_string(format!("{}/examples/{f}", env!("CARGO_MANIFEST_DIR")))
                    .unwrap();
            let a = analyze(&src);
            assert!(
                a.errors.is_empty() && a.warnings.is_empty(),
                "{f}: {:?} {:?}",
                a.errors,
                a.warnings
            );
        }
    }
}
