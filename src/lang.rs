//! Limbajul Cangurul: lexer, parser și compilator în instrucțiuni simple.
//!
//! Cuvintele-cheie sunt acceptate în română (cu sau fără diacritice),
//! engleză și rusă, indiferent de majuscule — exact ca în versiunea originală,
//! astfel încât fișierele `.cng` vechi rulează fără modificări.

use std::collections::HashMap;
use std::ops::Range;

/// Acțiunile elementare ale cangurului.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Un pas înainte, lăsând urmă.
    Step,
    /// Un pas înainte, fără urmă.
    Jump,
    /// Rotire cu 90° în sensul acelor ceasornicului.
    Rotate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CondKind {
    /// Cangurul stă cu fața spre marginea câmpului.
    Border,
    /// Nodul din fața cangurului este deja pe o linie (sau e marginea).
    Line,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cond {
    pub negated: bool,
    pub kind: CondKind,
}

/// Instrucțiuni pentru mașina virtuală (vezi `vm.rs`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instr {
    Act(Action),
    /// Pune un contor nou pe stiva de repetări.
    PushCount(u32),
    /// Dacă contorul din vârf e 0 îl scoate și sare la țintă, altfel îl scade.
    CountDown(usize),
    Jump(usize),
    /// Sare la țintă dacă condiția este falsă.
    JumpUnless(Cond, usize),
    Call(usize),
    Return,
    Halt,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub code: Vec<Instr>,
    /// Linia din sursă (de la 1) pentru fiecare instrucțiune.
    pub lines: Vec<usize>,
    pub entry: usize,
}

/// Tipul unui bloc care se închide cu SFÎRȘITUL.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    Procedure,
    Repeat,
    Cycle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// Un semn sau cuvânt-cheie pus unde nu-i este locul.
    UnexpectedToken(String),
    /// Un cuvânt care nu este comandă.
    UnknownWord(String),
    UnknownCondition(String),
    /// După DACĂ / ALTFEL sau la sfârșit lipsește comanda.
    ExpectedCommand,
    ExpectedNumber,
    ExpectedName,
    ExpectedCondition,
    /// Blocul deschis pe acest rând nu este închis.
    MissingEnd(BlockKind),
    WrongEnd {
        expected: BlockKind,
    },
    /// SFÎRȘITUL fără bloc deschis.
    StrayEnd,
    /// ALTFEL fără DACĂ.
    StrayElse,
    /// `]` fără `[`.
    StrayClose,
    UnknownProcedure(String),
    DuplicateProcedure(String),
    ProcedureInMain,
    CommandOutsideMain,
    MissingMainEnd,
    UnclosedComment,
    TextAfterProgram,
    NumberTooLarge,
}

/// O eroare găsită înainte de rulare.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompileError {
    pub line: usize,
    /// Poziția în text (octeți); poate fi goală la sfârșitul textului.
    pub span: Range<usize>,
    pub kind: ErrorKind,
    /// Cuvântul care probabil trebuia scris în loc (corectare rapidă).
    pub hint: Option<String>,
}

impl CompileError {
    fn new(line: usize, span: Range<usize>, kind: ErrorKind) -> CompileError {
        CompileError {
            line,
            span,
            kind,
            hint: None,
        }
    }
}

// ---------------------------------------------------------------- Lexer ----

#[derive(Clone, Debug, PartialEq, Eq)]
enum Tok {
    Word(String),
    Number(u64),
    Open,
    Close,
}

#[derive(Clone, Debug)]
struct Token {
    tok: Tok,
    /// Textul original, pentru mesaje de eroare.
    text: String,
    line: usize,
    span: Range<usize>,
}

/// Aduce un cuvânt la forma canonică: majuscule, fără diacritice.
pub fn normalize(word: &str) -> String {
    word.chars()
        .flat_map(char::to_uppercase)
        .map(|c| match c {
            'Ă' | 'Â' => 'A',
            'Î' => 'I',
            'Ș' | 'Ş' => 'S',
            'Ț' | 'Ţ' => 'T',
            'Ё' => 'Е',
            c => c,
        })
        .collect()
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn lex(src: &str, errors: &mut Vec<CompileError>) -> Vec<Token> {
    let mut out = Vec::new();
    let mut it = src.char_indices().peekable();
    let mut line = 1;
    while let Some(&(i, c)) = it.peek() {
        match c {
            '\n' => {
                line += 1;
                it.next();
            }
            c if c.is_whitespace() => {
                it.next();
            }
            '/' => {
                it.next();
                if it.peek().is_some_and(|&(_, c)| c == '/') {
                    while it.peek().is_some_and(|&(_, c)| c != '\n') {
                        it.next();
                    }
                } else {
                    errors.push(CompileError::new(
                        line,
                        i..i + 1,
                        ErrorKind::UnexpectedToken("/".into()),
                    ));
                }
            }
            // Comentariu pe mai multe rânduri: * ... *
            '*' => {
                let start = line;
                it.next();
                let mut closed = false;
                for (_, c) in it.by_ref() {
                    match c {
                        '*' => {
                            closed = true;
                            break;
                        }
                        '\n' => line += 1,
                        _ => {}
                    }
                }
                if !closed {
                    errors.push(CompileError::new(
                        start,
                        i..i + 1,
                        ErrorKind::UnclosedComment,
                    ));
                }
            }
            '[' | ']' => {
                it.next();
                let tok = if c == '[' { Tok::Open } else { Tok::Close };
                out.push(Token {
                    tok,
                    text: c.to_string(),
                    line,
                    span: i..i + 1,
                });
            }
            c if is_word_char(c) => {
                let mut end = i;
                while let Some(&(j, c)) = it.peek() {
                    if !is_word_char(c) {
                        break;
                    }
                    end = j + c.len_utf8();
                    it.next();
                }
                let text = &src[i..end];
                let tok = if text.chars().all(|c| c.is_ascii_digit()) {
                    // Numerele prea mari sunt semnalate de REPETĂ.
                    Tok::Number(text.parse().unwrap_or(u64::MAX))
                } else {
                    Tok::Word(normalize(text))
                };
                out.push(Token {
                    tok,
                    text: text.into(),
                    line,
                    span: i..end,
                });
            }
            other => {
                it.next();
                let span = i..i + other.len_utf8();
                errors.push(CompileError::new(
                    line,
                    span,
                    ErrorKind::UnexpectedToken(other.to_string()),
                ));
            }
        }
    }
    out
}

// ------------------------------------------------------------- Keywords ----

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kw {
    Step,
    Jump,
    Rotate,
    Repeat,
    Times,
    End,
    Procedure,
    Execute,
    While,
    If,
    Then,
    Else,
    Not,
    Border,
    Line,
}

fn keyword(w: &str) -> Option<Kw> {
    Some(match w {
        "PAS" | "STEP" | "ШАГ" => Kw::Step,
        "SALT" | "JUMP" | "ПРЫЖОК" => Kw::Jump,
        "ROTIRE" | "ROTATE" | "ПОВОРОТ" => Kw::Rotate,
        "REPETA" | "REPEAT" | "ПОВТОРИ" => Kw::Repeat,
        "ORI" | "TIMES" | "РАЗ" | "РАЗА" => Kw::Times,
        "SFIRSITUL" | "SFARSITUL" | "END" | "КОНЕЦ" => Kw::End,
        "PROCEDURA" | "PROCEDURE" | "ПРОЦЕДУРА" => Kw::Procedure,
        "EXECUTA" | "EXECUTE" | "ВЫПОЛНИТЬ" | "ВЫПОЛНИ" => Kw::Execute,
        "CIT" | "CAT" | "WHILE" | "ПОКА" => Kw::While,
        "DACA" | "IF" | "ЕСЛИ" => Kw::If,
        "ATUNCI" | "THEN" | "ТОГДА" => Kw::Then,
        "ALTFEL" | "ELSE" | "ИНАЧЕ" => Kw::Else,
        "NU" | "NOT" | "НЕ" => Kw::Not,
        "E_MARGINE" | "IS_BORDER" | "КРАЙ" | "E_MARGINEA" => Kw::Border,
        "E_LINIE" | "IS_LINE" | "ЛИНИЯ" => Kw::Line,
        _ => return None,
    })
}

/// Cuvintele cu care poate începe o comandă (pentru sugestii).
pub(crate) const COMMAND_WORDS: &[&str] = &[
    "PAS",
    "SALT",
    "ROTIRE",
    "REPETA",
    "CIT",
    "DACA",
    "ALTFEL",
    "EXECUTA",
    "PROCEDURA",
    "SFIRSITUL",
    "STEP",
    "JUMP",
    "ROTATE",
    "REPEAT",
    "WHILE",
    "IF",
    "ELSE",
    "EXECUTE",
    "PROCEDURE",
    "END",
    "ШАГ",
    "ПРЫЖОК",
    "ПОВОРОТ",
    "ПОВТОРИ",
    "ПОКА",
    "ЕСЛИ",
    "ИНАЧЕ",
    "ВЫПОЛНИТЬ",
    "ПРОЦЕДУРА",
    "КОНЕЦ",
];

pub(crate) const CONDITION_WORDS: &[&str] = &[
    "E_MARGINE",
    "E_LINIE",
    "IS_BORDER",
    "IS_LINE",
    "КРАЙ",
    "ЛИНИЯ",
];

/// Cuvântul care urmează după SFÎRȘITUL / END / КОНЕЦ.
fn end_kind(w: &str) -> Option<BlockKind> {
    Some(match w {
        "PROCEDURII" | "PROCEDURE" | "ПРОЦЕДУРЫ" => BlockKind::Procedure,
        "REPETARII" | "REPEAT" | "ПОВТОРА" | "ПОВОРА" | "ПОВТОРЕНИЯ" => {
            BlockKind::Repeat
        }
        "CICLULUI" | "CYCLE" | "WHILE" | "ЦИКЛА" => BlockKind::Cycle,
        _ => return None,
    })
}

// ------------------------------------------------------------------- AST ---

#[derive(Debug)]
pub(crate) struct Stmt {
    pub kind: StmtKind,
    pub line: usize,
    /// Poziția cuvântului-cheie care începe comanda.
    pub span: Range<usize>,
}

#[derive(Debug)]
pub(crate) enum StmtKind {
    Act(Action),
    Repeat {
        n: u32,
        n_span: Range<usize>,
        body: Vec<Stmt>,
    },
    While(Cond, Vec<Stmt>),
    If(Cond, Box<Stmt>, Option<Box<Stmt>>),
    /// Numele normalizat, textul original și poziția numelui.
    Call(String, String, Range<usize>),
    /// Loc rămas gol din cauza unei erori.
    Invalid,
}

#[derive(Debug)]
pub(crate) struct ProcDef {
    pub name: String,
    pub text: String,
    pub body: Vec<Stmt>,
    pub line: usize,
    pub span: Range<usize>,
}

#[derive(Debug)]
pub(crate) struct Ast {
    pub procs: Vec<ProcDef>,
    pub main: Vec<Stmt>,
    /// Poziția parantezei `[` (dacă există).
    pub main_at: Option<(usize, Range<usize>)>,
}

// ---------------------------------------------------------------- Parser ---

/// Parser care nu se oprește la prima greșeală: o notează, sare peste restul
/// rândului și continuă, ca analizorul să poată arăta toate erorile odată.
struct Parser<'a> {
    toks: Vec<Token>,
    pos: usize,
    end_line: usize,
    end_pos: usize,
    errors: &'a mut Vec<CompileError>,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Token> {
        self.toks.get(self.pos)
    }

    fn peek_kw(&self) -> Option<Kw> {
        match self.peek()?.tok {
            Tok::Word(ref w) => keyword(w),
            _ => None,
        }
    }

    fn eat_kw(&mut self, kw: Kw) -> bool {
        let found = self.peek_kw() == Some(kw);
        self.pos += usize::from(found);
        found
    }

    /// Rândul și poziția cuvântului curent (sau sfârșitul textului).
    fn here(&self) -> (usize, Range<usize>) {
        self.peek()
            .map_or((self.end_line, self.end_pos..self.end_pos), |t| {
                (t.line, t.span.clone())
            })
    }

    fn error(&mut self, line: usize, span: Range<usize>, kind: ErrorKind, hint: Option<String>) {
        self.errors.push(CompileError {
            line,
            span,
            kind,
            hint,
        });
    }

    fn error_here(&mut self, kind: ErrorKind) {
        let (line, span) = self.here();
        self.error(line, span, kind, None);
    }

    /// Cuvinte la care se oprește un bloc.
    fn at_block_end(&self) -> bool {
        match self.peek() {
            None => true,
            Some(t) if matches!(t.tok, Tok::Open | Tok::Close) => true,
            _ => matches!(self.peek_kw(), Some(Kw::End | Kw::Procedure)),
        }
    }

    /// După o eroare: sare peste cuvântul greșit și restul rândului,
    /// dar nu peste cuvintele care închid blocuri.
    fn recover(&mut self, before: usize) {
        if self.pos == before && !self.at_block_end() {
            self.pos += 1;
        }
        let Some(line) = self.pos.checked_sub(1).map(|i| self.toks[i].line) else {
            return;
        };
        while self.peek().is_some_and(|t| t.line == line) && !self.at_block_end() {
            self.pos += 1;
        }
    }

    fn cond(&mut self) -> Cond {
        let negated = self.eat_kw(Kw::Not);
        let kind = match self.peek_kw() {
            Some(Kw::Border) => CondKind::Border,
            Some(Kw::Line) => CondKind::Line,
            _ => {
                match self.peek().cloned() {
                    Some(Token {
                        tok: Tok::Word(w),
                        text,
                        line,
                        span,
                    }) if keyword(&w).is_none() => {
                        let hint = crate::check::suggest(&w, &text, CONDITION_WORDS);
                        self.error(line, span, ErrorKind::UnknownCondition(text), hint);
                        self.pos += 1;
                    }
                    _ => self.error_here(ErrorKind::ExpectedCondition),
                }
                return Cond {
                    negated,
                    kind: CondKind::Border,
                };
            }
        };
        self.pos += 1;
        Cond { negated, kind }
    }

    /// Citește `SFÎRȘITUL <tip>`; tipul poate lipsi.
    fn end(&mut self, kind: BlockKind, open_line: usize, open_span: &Range<usize>) {
        if !self.eat_kw(Kw::End) {
            self.error(
                open_line,
                open_span.clone(),
                ErrorKind::MissingEnd(kind),
                None,
            );
            return;
        }
        if let Some(Token {
            tok: Tok::Word(w),
            line,
            span,
            ..
        }) = self.peek().cloned()
            && let Some(found) = end_kind(&w)
        {
            if found != kind {
                self.error(line, span, ErrorKind::WrongEnd { expected: kind }, None);
            }
            self.pos += 1;
        }
    }

    /// SFÎRȘITUL rătăcit: îl raportează și îl sare împreună cu tipul lui.
    fn stray_end(&mut self) {
        self.error_here(ErrorKind::StrayEnd);
        self.pos += 1;
        if let Some(Token {
            tok: Tok::Word(w), ..
        }) = self.peek()
            && end_kind(w).is_some()
        {
            self.pos += 1;
        }
    }

    /// Instrucțiuni până la SFÎRȘITUL, PROCEDURA, `[` sau `]`.
    fn block(&mut self) -> Vec<Stmt> {
        let mut out = Vec::new();
        while !self.at_block_end() {
            let before = self.pos;
            match self.stmt() {
                Some(s) => out.push(s),
                None => self.recover(before),
            }
        }
        out
    }

    /// Comanda de după DACĂ sau ALTFEL.
    fn branch(&mut self) -> Box<Stmt> {
        let (line, span) = self.here();
        let before = self.pos;
        Box::new(self.stmt().unwrap_or_else(|| {
            self.recover(before);
            Stmt {
                kind: StmtKind::Invalid,
                line,
                span,
            }
        }))
    }

    fn stmt(&mut self) -> Option<Stmt> {
        let Some(t) = self.peek().cloned() else {
            self.error_here(ErrorKind::ExpectedCommand);
            return None;
        };
        let Token { line, span, .. } = t;
        let kw = match &t.tok {
            Tok::Word(w) => match keyword(w) {
                Some(kw) => kw,
                None => {
                    let hint = crate::check::suggest(w, &t.text, COMMAND_WORDS);
                    self.error(line, span, ErrorKind::UnknownWord(t.text), hint);
                    return None;
                }
            },
            Tok::Number(_) => {
                self.error(line, span, ErrorKind::UnexpectedToken(t.text), None);
                return None;
            }
            Tok::Open | Tok::Close => {
                self.error_here(ErrorKind::ExpectedCommand);
                return None;
            }
        };
        let kind = match kw {
            Kw::End | Kw::Procedure => {
                self.error_here(ErrorKind::ExpectedCommand);
                return None;
            }
            Kw::Else => {
                self.error(line, span, ErrorKind::StrayElse, None);
                return None;
            }
            Kw::Times | Kw::Then | Kw::Not | Kw::Border | Kw::Line => {
                self.error(line, span, ErrorKind::UnexpectedToken(t.text), None);
                return None;
            }
            Kw::Step => StmtKind::Act(Action::Step),
            Kw::Jump => StmtKind::Act(Action::Jump),
            Kw::Rotate => StmtKind::Act(Action::Rotate),
            Kw::Repeat => {
                self.pos += 1;
                let (n, n_span) = match self.peek().cloned() {
                    Some(Token {
                        tok: Tok::Number(n),
                        span: n_span,
                        ..
                    }) => {
                        self.pos += 1;
                        let n = u32::try_from(n).unwrap_or_else(|_| {
                            self.error(line, n_span.clone(), ErrorKind::NumberTooLarge, None);
                            1
                        });
                        (n, n_span)
                    }
                    _ => {
                        self.error_here(ErrorKind::ExpectedNumber);
                        // Sare peste ce a scris elevul în loc de număr, până la ORI.
                        while let Some(Token {
                            tok: Tok::Word(w),
                            line: l,
                            ..
                        }) = self.peek()
                            && *l == line
                            && keyword(w).is_none()
                        {
                            self.pos += 1;
                        }
                        // „REPETĂ de 3 ORI”: numărul de după cuvântul în plus e folosit.
                        match self.peek().cloned() {
                            Some(Token {
                                tok: Tok::Number(n),
                                span: n_span,
                                line: l,
                                ..
                            }) if l == line => {
                                self.pos += 1;
                                (u32::try_from(n).unwrap_or(1), n_span)
                            }
                            _ => (1, span.clone()),
                        }
                    }
                };
                self.eat_kw(Kw::Times);
                let body = self.block();
                self.end(BlockKind::Repeat, line, &span);
                return Some(Stmt {
                    kind: StmtKind::Repeat { n, n_span, body },
                    line,
                    span,
                });
            }
            Kw::While => {
                self.pos += 1;
                let c = self.cond();
                let body = self.block();
                self.end(BlockKind::Cycle, line, &span);
                return Some(Stmt {
                    kind: StmtKind::While(c, body),
                    line,
                    span,
                });
            }
            Kw::If => {
                self.pos += 1;
                let c = self.cond();
                self.eat_kw(Kw::Then);
                let then = self.branch();
                let other = self.eat_kw(Kw::Else).then(|| self.branch());
                return Some(Stmt {
                    kind: StmtKind::If(c, then, other),
                    line,
                    span,
                });
            }
            Kw::Execute => {
                self.pos += 1;
                return match self.peek().cloned() {
                    Some(Token {
                        tok: Tok::Word(w),
                        text,
                        span: name_span,
                        ..
                    }) if keyword(&w).is_none() => {
                        self.pos += 1;
                        Some(Stmt {
                            kind: StmtKind::Call(w, text, name_span),
                            line,
                            span,
                        })
                    }
                    _ => {
                        self.error_here(ErrorKind::ExpectedName);
                        None
                    }
                };
            }
        };
        self.pos += 1;
        Some(Stmt { kind, line, span })
    }

    fn procedure(&mut self, procs: &mut Vec<ProcDef>) {
        let Token { line, span, .. } = self.toks[self.pos].clone();
        self.pos += 1;
        let name = match self.peek().cloned() {
            Some(Token {
                tok: Tok::Word(w),
                text,
                span: name_span,
                line: name_line,
            }) if keyword(&w).is_none() && name_line == line => {
                self.pos += 1;
                if procs.iter().any(|p| p.name == w) {
                    self.error(
                        line,
                        name_span.clone(),
                        ErrorKind::DuplicateProcedure(text.clone()),
                        None,
                    );
                }
                Some((w, text, name_span))
            }
            _ => {
                self.error_here(ErrorKind::ExpectedName);
                None
            }
        };
        let body = self.block();
        self.end(BlockKind::Procedure, line, &span);
        if let Some((name, text, span)) = name
            && !procs.iter().any(|p| p.name == name)
        {
            procs.push(ProcDef {
                name,
                text,
                body,
                line,
                span,
            });
        }
    }

    /// Conținutul dintre `[` și `]`.
    fn main_body(&mut self, open: &Token, procs: &mut Vec<ProcDef>) -> Vec<Stmt> {
        let mut body = Vec::new();
        loop {
            body.extend(self.block());
            match self.peek().cloned() {
                Some(Token {
                    tok: Tok::Close, ..
                }) => {
                    self.pos += 1;
                    return body;
                }
                Some(Token {
                    tok: Tok::Open,
                    line,
                    span,
                    ..
                }) => {
                    self.error(line, span, ErrorKind::UnexpectedToken("[".into()), None);
                    self.pos += 1;
                }
                Some(t) if self.peek_kw() == Some(Kw::Procedure) => {
                    self.error(t.line, t.span, ErrorKind::ProcedureInMain, None);
                    self.procedure(procs);
                }
                Some(_) => self.stray_end(),
                None => {
                    self.error(
                        open.line,
                        open.span.clone(),
                        ErrorKind::MissingMainEnd,
                        None,
                    );
                    return body;
                }
            }
        }
    }

    fn program(&mut self) -> Ast {
        let mut procs = Vec::new();
        let mut main: Option<Vec<Stmt>> = None;
        let mut main_at = None;
        let mut loose: Vec<Stmt> = Vec::new();
        let mut reported_after_main = false;

        while let Some(t) = self.peek().cloned() {
            if main.is_some() && !reported_after_main {
                self.error(t.line, t.span.clone(), ErrorKind::TextAfterProgram, None);
                reported_after_main = true;
            }
            match t.tok {
                Tok::Open => {
                    self.pos += 1;
                    let body = self.main_body(&t, &mut procs);
                    if main.is_none() {
                        main = Some(body);
                        main_at = Some((t.line, t.span.clone()));
                    }
                }
                Tok::Close => {
                    self.error(t.line, t.span, ErrorKind::StrayClose, None);
                    self.pos += 1;
                }
                _ => match self.peek_kw() {
                    Some(Kw::Procedure) => self.procedure(&mut procs),
                    Some(Kw::End) => self.stray_end(),
                    _ => {
                        let before = self.pos;
                        match self.stmt() {
                            Some(s) if main.is_none() => loose.push(s),
                            Some(_) => {}
                            None => self.recover(before),
                        }
                    }
                },
            }
        }

        let main = match main {
            Some(m) => {
                // Programul fără [ ] e permis, dar comenzile libere înainte de [ nu.
                if let Some(s) = loose.first() {
                    self.error(s.line, s.span.clone(), ErrorKind::CommandOutsideMain, None);
                }
                m
            }
            None => loose,
        };
        Ast {
            procs,
            main,
            main_at,
        }
    }
}

/// Rezultatul analizei: toate erorile, avertismentele și programul (dacă nu are erori).
#[derive(Debug, Default)]
pub struct Analysis {
    pub errors: Vec<CompileError>,
    pub warnings: Vec<crate::check::Warning>,
    pub program: Option<Program>,
}

/// Analizează textul: găsește toate greșelile și, dacă nu sunt, îl compilează.
pub fn analyze(src: &str) -> Analysis {
    let mut errors = Vec::new();
    let toks = lex(src, &mut errors);
    let mut p = Parser {
        toks,
        pos: 0,
        end_line: src.lines().count().max(1),
        end_pos: src.trim_end().len(),
        errors: &mut errors,
    };
    let ast = p.program();
    let warnings = crate::check::check(&ast, &mut errors);

    // Un comentariu neînchis ascunde restul textului: blocurile „neînchise” sunt doar urmări ale lui.
    if errors.iter().any(|e| e.kind == ErrorKind::UnclosedComment) {
        errors.retain(|e| !matches!(e.kind, ErrorKind::MissingEnd(_) | ErrorKind::MissingMainEnd));
    }
    errors.sort_by_key(|e| (e.span.start, e.line));
    errors.dedup_by(|a, b| a.span == b.span && a.line == b.line);
    let program = errors.is_empty().then(|| generate(&ast, src));
    Analysis {
        errors,
        warnings,
        program,
    }
}

/// Compilează textul unui program Cangurul; întoarce prima eroare.
#[cfg(test)]
pub fn compile(src: &str) -> Result<Program, CompileError> {
    let mut a = analyze(src);
    a.program.ok_or_else(|| a.errors.swap_remove(0))
}

// -------------------------------------------------------------- Compiler ---

struct Compiler<'a> {
    code: Vec<Instr>,
    lines: Vec<usize>,
    procs: &'a HashMap<String, usize>,
    /// (index instrucțiune Call, nume procedură) — completate la final.
    calls: Vec<(usize, String)>,
}

impl Compiler<'_> {
    fn emit(&mut self, i: Instr, line: usize) -> usize {
        self.code.push(i);
        self.lines.push(line);
        self.code.len() - 1
    }

    fn patch(&mut self, at: usize, target: usize) {
        match &mut self.code[at] {
            Instr::CountDown(t) | Instr::Jump(t) | Instr::JumpUnless(_, t) => *t = target,
            _ => unreachable!(),
        }
    }

    fn stmts(&mut self, body: &[Stmt]) {
        body.iter().for_each(|s| self.stmt(s));
    }

    fn stmt(&mut self, s: &Stmt) {
        let line = s.line;
        match &s.kind {
            StmtKind::Act(a) => {
                self.emit(Instr::Act(*a), line);
            }
            StmtKind::Repeat { n, body, .. } => {
                self.emit(Instr::PushCount(*n), line);
                let top = self.emit(Instr::CountDown(0), line);
                self.stmts(body);
                self.emit(Instr::Jump(top), line);
                let end = self.code.len();
                self.patch(top, end);
            }
            StmtKind::While(c, body) => {
                let top = self.emit(Instr::JumpUnless(*c, 0), line);
                self.stmts(body);
                self.emit(Instr::Jump(top), line);
                let end = self.code.len();
                self.patch(top, end);
            }
            StmtKind::If(c, then, other) => {
                let test = self.emit(Instr::JumpUnless(*c, 0), line);
                self.stmt(then);
                if let Some(other) = other {
                    let skip = self.emit(Instr::Jump(0), line);
                    let else_at = self.code.len();
                    self.patch(test, else_at);
                    self.stmt(other);
                    let end = self.code.len();
                    self.patch(skip, end);
                } else {
                    let end = self.code.len();
                    self.patch(test, end);
                }
            }
            StmtKind::Call(name, ..) => {
                debug_assert!(self.procs.contains_key(name));
                let at = self.emit(Instr::Call(0), line);
                self.calls.push((at, name.clone()));
            }
            StmtKind::Invalid => {}
        }
    }
}

/// Generează codul pentru un program fără erori.
fn generate(ast: &Ast, src: &str) -> Program {
    let last_line = src.lines().count().max(1);
    let index: HashMap<String, usize> = ast
        .procs
        .iter()
        .enumerate()
        .map(|(i, p)| (p.name.clone(), i))
        .collect();
    let mut c = Compiler {
        code: Vec::new(),
        lines: Vec::new(),
        procs: &index,
        calls: Vec::new(),
    };

    c.stmts(&ast.main);
    c.emit(Instr::Halt, last_line);

    let mut starts = Vec::new();
    for p in &ast.procs {
        starts.push(c.code.len());
        c.stmts(&p.body);
        c.emit(Instr::Return, p.line);
    }
    for (at, name) in std::mem::take(&mut c.calls) {
        c.code[at] = Instr::Call(starts[index[&name]]);
    }
    Program {
        code: c.code,
        lines: c.lines,
        entry: 0,
    }
}

// ------------------------------------------------------- Syntax colouring ---

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenClass {
    Plain,
    Action,
    Control,
    Condition,
    Number,
    Bracket,
    Comment,
}

/// Împarte textul în segmente colorate pentru editor (nu dă erori).
pub fn highlight(src: &str) -> Vec<(std::ops::Range<usize>, TokenClass)> {
    let mut out = Vec::new();
    let mut it = src.char_indices().peekable();
    while let Some((i, c)) = it.next() {
        let mut end = i + c.len_utf8();
        let class = if c == '/' && it.peek().is_some_and(|&(_, c)| c == '/') {
            while let Some(&(j, c)) = it.peek() {
                if c == '\n' {
                    break;
                }
                end = j + c.len_utf8();
                it.next();
            }
            TokenClass::Comment
        } else if c == '*' {
            for (j, c) in it.by_ref() {
                end = j + c.len_utf8();
                if c == '*' {
                    break;
                }
            }
            TokenClass::Comment
        } else if is_word_char(c) {
            while let Some(&(j, c)) = it.peek() {
                if !is_word_char(c) {
                    break;
                }
                end = j + c.len_utf8();
                it.next();
            }
            let word = &src[i..end];
            if word.chars().all(|c| c.is_ascii_digit()) {
                TokenClass::Number
            } else {
                let w = normalize(word);
                match keyword(&w) {
                    Some(Kw::Step | Kw::Jump | Kw::Rotate) => TokenClass::Action,
                    Some(Kw::Border | Kw::Line | Kw::Not) => TokenClass::Condition,
                    Some(_) => TokenClass::Control,
                    None if end_kind(&w).is_some() => TokenClass::Control,
                    None => TokenClass::Plain,
                }
            }
        } else if c == '[' || c == ']' {
            TokenClass::Bracket
        } else {
            TokenClass::Plain
        };
        out.push((i..end, class));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diacritics_and_case() {
        assert_eq!(normalize("Repetă"), "REPETA");
        assert_eq!(normalize("sfîrșitul"), "SFIRSITUL");
        assert_eq!(normalize("Cât"), "CAT");
        assert_eq!(normalize("шаг"), "ШАГ");
    }

    #[test]
    fn simple_program() {
        let p = compile("[\npas\nsalt\nrotire\n]").unwrap();
        assert_eq!(
            p.code,
            vec![
                Instr::Act(Action::Step),
                Instr::Act(Action::Jump),
                Instr::Act(Action::Rotate),
                Instr::Halt
            ]
        );
        assert_eq!(&p.lines[..3], &[2, 3, 4]);
    }

    #[test]
    fn errors_have_lines() {
        let e = compile("[\npas\nzbor\n]").unwrap_err();
        assert_eq!(e.line, 3);
        assert_eq!(e.kind, ErrorKind::UnknownWord("zbor".into()));

        let e = compile("[\nexecuta X\n]").unwrap_err();
        assert_eq!(e.kind, ErrorKind::UnknownProcedure("X".into()));

        let e = compile("[\nrepeta 3 ori\npas\nsfirsitul ciclului\n]").unwrap_err();
        assert_eq!(
            e.kind,
            ErrorKind::WrongEnd {
                expected: BlockKind::Repeat
            }
        );

        let e = compile("[\nrepeta 3 ori\npas\n]").unwrap_err();
        assert_eq!(
            (e.line, e.kind),
            (2, ErrorKind::MissingEnd(BlockKind::Repeat))
        );

        let e = compile("[\n* neinchis\npas\n]").unwrap_err();
        assert_eq!((e.line, e.kind), (2, ErrorKind::UnclosedComment));
    }

    #[test]
    fn without_brackets() {
        assert!(compile("pas pas rotire").is_ok());
    }
}
