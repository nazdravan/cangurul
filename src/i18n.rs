//! Textele interfeței în română, rusă și engleză.

use crate::check::WarningKind;
use crate::lang::{BlockKind, ErrorKind};
use crate::vm::RuntimeError;
use crate::world::Dir;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Lang {
    Ro,
    Ru,
    En,
}

impl Lang {
    pub const ALL: [Lang; 3] = [Lang::Ro, Lang::Ru, Lang::En];

    pub fn name(self) -> &'static str {
        match self {
            Lang::Ro => "Română",
            Lang::Ru => "Русский",
            Lang::En => "English",
        }
    }

    pub fn t(self) -> &'static Strings {
        match self {
            Lang::Ro => &RO,
            Lang::Ru => &RU,
            Lang::En => &EN,
        }
    }
}

/// Cuvintele-cheie folosite la inserarea șabloanelor și în ajutor.
pub struct Keywords {
    pub step: &'static str,
    pub jump: &'static str,
    pub rotate: &'static str,
    pub repeat: &'static str,
    pub times: &'static str,
    pub end_repeat: &'static str,
    pub while_: &'static str,
    pub end_cycle: &'static str,
    pub if_: &'static str,
    pub else_: &'static str,
    pub not: &'static str,
    pub border: &'static str,
    pub line: &'static str,
    pub procedure: &'static str,
    pub end_procedure: &'static str,
    pub execute: &'static str,
    pub name: &'static str,
}

pub struct Strings {
    pub kw: Keywords,
    pub title: &'static str,
    pub new: &'static str,
    pub open: &'static str,
    pub save: &'static str,
    pub save_as: &'static str,
    pub examples: &'static str,
    pub help: &'static str,
    pub settings: &'static str,
    pub theme: &'static str,
    pub theme_system: &'static str,
    pub theme_light: &'static str,
    pub theme_dark: &'static str,
    pub run: &'static str,
    pub step_once: &'static str,
    pub pause: &'static str,
    pub stop: &'static str,
    pub reset: &'static str,
    pub check: &'static str,
    pub format: &'static str,
    pub speed: &'static str,
    pub instant: &'static str,
    pub field: &'static str,
    pub columns: &'static str,
    pub rows: &'static str,
    pub show_jumps: &'static str,
    pub show_grid: &'static str,
    pub program: &'static str,
    pub untitled: &'static str,
    pub ready: &'static str,
    pub running: &'static str,
    pub paused: &'static str,
    pub finished: &'static str,
    pub stopped: &'static str,
    pub no_errors: &'static str,
    pub error_line: &'static str,
    pub position: &'static str,
    pub direction: &'static str,
    pub actions: &'static str,
    pub lines_drawn: &'static str,
    pub dirs: [&'static str; 4],
    pub unsaved_title: &'static str,
    pub unsaved_text: &'static str,
    pub discard: &'static str,
    pub cancel: &'static str,
    pub file_error: &'static str,
    pub cng_files: &'static str,
    pub problems: &'static str,
    pub no_problems: &'static str,
    pub live_check: &'static str,
    pub did_you_mean: &'static str,
    pub fix: &'static str,
    pub line_short: &'static str,
    pub summary: fn(usize, usize) -> String,
    pub help_text: &'static [(&'static str, &'static str)],
    pub about: &'static str,
    pub shortcuts: &'static str,
}

impl Strings {
    pub fn dir(&self, d: Dir) -> &'static str {
        self.dirs[match d {
            Dir::East => 0,
            Dir::South => 1,
            Dir::West => 2,
            Dir::North => 3,
        }]
    }

    fn block_end(&self, b: BlockKind) -> &'static str {
        match b {
            BlockKind::Procedure => self.kw.end_procedure,
            BlockKind::Repeat => self.kw.end_repeat,
            BlockKind::Cycle => self.kw.end_cycle,
        }
    }

    fn block_start(&self, b: BlockKind) -> &'static str {
        match b {
            BlockKind::Procedure => self.kw.procedure,
            BlockKind::Repeat => self.kw.repeat,
            BlockKind::Cycle => self.kw.while_,
        }
    }

    pub fn compile_error(&self, lang: Lang, e: &ErrorKind) -> String {
        use ErrorKind::*;
        let k = &self.kw;
        match lang {
            Lang::Ro => match e {
                UnexpectedToken(t) => format!("„{t}” nu are ce căuta aici"),
                UnknownWord(t) => format!("Cuvânt necunoscut „{t}”"),
                UnknownCondition(t) => format!(
                    "Condiție necunoscută „{t}” (se poate {} sau {})",
                    k.border, k.line
                ),
                ExpectedCommand => "Aici lipsește o comandă".into(),
                ExpectedNumber => format!("După {} trebuie un număr", k.repeat),
                ExpectedName => "Lipsește numele procedurii".into(),
                ExpectedCondition => format!("Lipsește condiția ({} sau {})", k.border, k.line),
                MissingEnd(b) => format!(
                    "„{}” nu este închis cu „{}”",
                    self.block_start(*b),
                    self.block_end(*b)
                ),
                WrongEnd { expected } => format!("Aici trebuia „{}”", self.block_end(*expected)),
                StrayEnd => "„SFÎRȘITUL” nu închide niciun bloc".into(),
                StrayElse => format!(
                    "„{}” fără „{}” (după {} poate fi o singură comandă)",
                    k.else_, k.if_, k.if_
                ),
                StrayClose => "„]” fără „[”".into(),
                UnknownProcedure(n) => format!("Procedura „{n}” nu există"),
                DuplicateProcedure(n) => format!("Procedura „{n}” este definită de două ori"),
                ProcedureInMain => {
                    "Procedurile se scriu înainte de programul principal [ … ]".into()
                }
                CommandOutsideMain => "Comenzile trebuie scrise între [ și ]".into(),
                MissingMainEnd => "„[” nu este închis cu „]”".into(),
                UnclosedComment => "Comentariul * … * nu este închis".into(),
                TextAfterProgram => "Text în plus după programul principal [ … ]".into(),
                NumberTooLarge => "Numărul este prea mare".into(),
            },
            Lang::Ru => match e {
                UnexpectedToken(t) => format!("«{t}» здесь не к месту"),
                UnknownWord(t) => format!("Неизвестное слово «{t}»"),
                UnknownCondition(t) => format!(
                    "Неизвестное условие «{t}» (можно {} или {})",
                    k.border, k.line
                ),
                ExpectedCommand => "Здесь не хватает команды".into(),
                ExpectedNumber => format!("После {} нужно число", k.repeat),
                ExpectedName => "Не указано имя процедуры".into(),
                ExpectedCondition => format!("Нет условия ({} или {})", k.border, k.line),
                MissingEnd(b) => format!(
                    "«{}» не закрыт словами «{}»",
                    self.block_start(*b),
                    self.block_end(*b)
                ),
                WrongEnd { expected } => {
                    format!("Здесь должно быть «{}»", self.block_end(*expected))
                }
                StrayEnd => "«КОНЕЦ» не закрывает ни одного блока".into(),
                StrayElse => format!(
                    "«{}» без «{}» (после {} может быть только одна команда)",
                    k.else_, k.if_, k.if_
                ),
                StrayClose => "«]» без «[»".into(),
                UnknownProcedure(n) => format!("Процедура «{n}» не найдена"),
                DuplicateProcedure(n) => format!("Процедура «{n}» описана дважды"),
                ProcedureInMain => "Процедуры пишутся до основной программы [ … ]".into(),
                CommandOutsideMain => "Команды нужно писать между [ и ]".into(),
                MissingMainEnd => "«[» не закрыта «]»".into(),
                UnclosedComment => "Комментарий * … * не закрыт".into(),
                TextAfterProgram => "Лишний текст после основной программы [ … ]".into(),
                NumberTooLarge => "Слишком большое число".into(),
            },
            Lang::En => match e {
                UnexpectedToken(t) => format!("\"{t}\" does not belong here"),
                UnknownWord(t) => format!("Unknown word \"{t}\""),
                UnknownCondition(t) => {
                    format!("Unknown condition \"{t}\" (use {} or {})", k.border, k.line)
                }
                ExpectedCommand => "A command is missing here".into(),
                ExpectedNumber => format!("{} must be followed by a number", k.repeat),
                ExpectedName => "Missing procedure name".into(),
                ExpectedCondition => format!("Missing condition ({} or {})", k.border, k.line),
                MissingEnd(b) => format!(
                    "\"{}\" is not closed with \"{}\"",
                    self.block_start(*b),
                    self.block_end(*b)
                ),
                WrongEnd { expected } => format!("Expected \"{}\" here", self.block_end(*expected)),
                StrayEnd => "\"END\" does not close any block".into(),
                StrayElse => format!(
                    "\"{}\" without \"{}\" ({} takes a single command)",
                    k.else_, k.if_, k.if_
                ),
                StrayClose => "\"]\" without \"[\"".into(),
                UnknownProcedure(n) => format!("Procedure \"{n}\" does not exist"),
                DuplicateProcedure(n) => format!("Procedure \"{n}\" is defined twice"),
                ProcedureInMain => "Procedures are written before the main program [ … ]".into(),
                CommandOutsideMain => "Commands must be written between [ and ]".into(),
                MissingMainEnd => "\"[\" is not closed with \"]\"".into(),
                UnclosedComment => "Comment * … * is not closed".into(),
                TextAfterProgram => "Extra text after the main program [ … ]".into(),
                NumberTooLarge => "Number is too large".into(),
            },
        }
    }

    pub fn warning(&self, lang: Lang, w: &WarningKind) -> String {
        use WarningKind::*;
        let k = &self.kw;
        match lang {
            Lang::Ro => match w {
                UnusedProcedure(n) => format!("Procedura „{n}” nu este folosită niciodată"),
                EmptyBlock => "Blocul nu conține nicio comandă".into(),
                RepeatZero => format!(
                    "{} 0 {}: comenzile nu se execută niciodată",
                    k.repeat, k.times
                ),
                LoopWithoutMovement => format!(
                    "Ciclul {} nu mută cangurul și poate rula la nesfârșit",
                    k.while_
                ),
                EndlessRecursion(n) => {
                    format!("Procedura „{n}” se apelează pe ea însăși fără oprire")
                }
                EmptyProgram => "Programul principal nu conține nicio comandă".into(),
            },
            Lang::Ru => match w {
                UnusedProcedure(n) => format!("Процедура «{n}» нигде не используется"),
                EmptyBlock => "В блоке нет ни одной команды".into(),
                RepeatZero => format!("{} 0 {}: команды никогда не выполнятся", k.repeat, k.times),
                LoopWithoutMovement => format!(
                    "Цикл {} не двигает кенгуренка и может не закончиться",
                    k.while_
                ),
                EndlessRecursion(n) => format!("Процедура «{n}» вызывает себя без остановки"),
                EmptyProgram => "В основной программе нет команд".into(),
            },
            Lang::En => match w {
                UnusedProcedure(n) => format!("Procedure \"{n}\" is never used"),
                EmptyBlock => "The block contains no commands".into(),
                RepeatZero => format!("{} 0 {}: the commands never run", k.repeat, k.times),
                LoopWithoutMovement => format!(
                    "The {} loop does not move the kangaroo and may never end",
                    k.while_
                ),
                EndlessRecursion(n) => format!("Procedure \"{n}\" calls itself without stopping"),
                EmptyProgram => "The main program has no commands".into(),
            },
        }
    }

    pub fn runtime_error(&self, lang: Lang, e: RuntimeError) -> &'static str {
        match (lang, e) {
            (Lang::Ro, RuntimeError::OutOfField) => "Cangurul nu poate ieși din câmp!",
            (Lang::Ro, RuntimeError::StackOverflow) => {
                "Stiva este plină (procedura se apelează la nesfârșit)"
            }
            (Lang::Ru, RuntimeError::OutOfField) => "Кенгуренок не может выйти за край поля!",
            (Lang::Ru, RuntimeError::StackOverflow) => {
                "Переполнение стека (процедура вызывает себя бесконечно)"
            }
            (Lang::En, RuntimeError::OutOfField) => "The kangaroo cannot leave the field!",
            (Lang::En, RuntimeError::StackOverflow) => {
                "Stack overflow (a procedure calls itself forever)"
            }
        }
    }
}

pub static RO: Strings = Strings {
    kw: Keywords {
        step: "PAS",
        jump: "SALT",
        rotate: "ROTIRE",
        repeat: "REPETĂ",
        times: "ORI",
        end_repeat: "SFÎRȘITUL REPETĂRII",
        while_: "CÎT",
        end_cycle: "SFÎRȘITUL CICLULUI",
        if_: "DACĂ",
        else_: "ALTFEL",
        not: "NU",
        border: "E_MARGINE",
        line: "E_LINIE",
        procedure: "PROCEDURA",
        end_procedure: "SFÎRȘITUL PROCEDURII",
        execute: "EXECUTĂ",
        name: "Nume",
    },
    title: "Cangurul",
    new: "Nou",
    open: "Deschide…",
    save: "Salvează",
    save_as: "Salvează ca…",
    examples: "Exemple",
    help: "Ajutor",
    settings: "Setări",
    theme: "Tema",
    theme_system: "Ca sistemul",
    theme_light: "Luminoasă",
    theme_dark: "Întunecată",
    run: "Execută",
    step_once: "Pas cu pas",
    pause: "Pauză",
    stop: "Stop",
    reset: "Resetare",
    check: "Verifică",
    format: "Formatează",
    speed: "Viteza",
    instant: "Instant",
    field: "Câmpul",
    columns: "Coloane",
    rows: "Rânduri",
    show_jumps: "Arată salturile",
    show_grid: "Arată grila",
    program: "Program",
    untitled: "fără nume",
    ready: "Gata de lucru",
    running: "Se execută…",
    paused: "Pauză",
    finished: "Executarea completă",
    stopped: "Executarea a fost oprită",
    no_errors: "Programul nu conține erori",
    error_line: "Eroare în rândul",
    position: "Poziția",
    direction: "Direcția",
    actions: "Comenzi executate",
    lines_drawn: "Linii",
    dirs: ["→ est", "↓ sud", "← vest", "↑ nord"],
    unsaved_title: "Modificări nesalvate",
    unsaved_text: "Programul are modificări nesalvate. Le salvați?",
    discard: "Nu salva",
    cancel: "Anulare",
    file_error: "Nu pot deschide/salva fișierul",
    cng_files: "Programe Cangurul",
    problems: "Probleme",
    no_problems: "Nicio problemă găsită",
    live_check: "Verifică programul în timp ce scriu",
    did_you_mean: "Poate ați vrut",
    fix: "Corectează",
    line_short: "rândul",
    summary: |e, w| match (e, w) {
        (0, 0) => "Programul nu conține erori".into(),
        (0, w) => format!("Fără erori · avertismente: {w}"),
        (e, 0) => format!("Erori: {e}"),
        (e, w) => format!("Erori: {e} · avertismente: {w}"),
    },
    help_text: &[
        (
            "Câmpul",
            "Cangurul se mișcă pe nodurile unei grile. La început stă în colțul din stânga-sus și privește spre dreapta.",
        ),
        (
            "PAS",
            "Cangurul face un pas înainte și lasă o urmă (o linie).",
        ),
        ("SALT", "Cangurul face un pas înainte fără să lase urmă."),
        (
            "ROTIRE",
            "Cangurul se rotește cu 90° în sensul acelor ceasornicului.",
        ),
        (
            "REPETĂ N ORI … SFÎRȘITUL REPETĂRII",
            "Repetă comenzile dintre ele de N ori.",
        ),
        (
            "CÎT [NU] condiție … SFÎRȘITUL CICLULUI",
            "Repetă comenzile cât timp condiția este adevărată.",
        ),
        (
            "DACĂ [NU] condiție comandă1 ALTFEL comandă2",
            "Execută comanda1 dacă condiția e adevărată, altfel comanda2 (partea ALTFEL poate lipsi).",
        ),
        (
            "E_MARGINE",
            "Adevărat când cangurul stă cu fața spre marginea câmpului.",
        ),
        (
            "E_LINIE",
            "Adevărat când în fața cangurului este deja o linie (sau marginea).",
        ),
        (
            "PROCEDURA Nume … SFÎRȘITUL PROCEDURII",
            "Definește o procedură. Procedurile se scriu înainte de programul principal.",
        ),
        ("EXECUTĂ Nume", "Execută procedura cu numele dat."),
        (
            "[ … ]",
            "Programul principal se scrie între paranteze pătrate.",
        ),
        (
            "// comentariu   * comentariu *",
            "Textul după // până la sfârșitul rândului sau dintre două * este ignorat.",
        ),
        (
            "Verificare",
            "Programul este verificat în timp ce scrieți: greșelile sunt subliniate cu roșu, avertismentele cu galben. Lista „Probleme” arată toate greșelile; un click duce la rândul respectiv, iar „Corectează” înlocuiește cuvântul greșit.",
        ),
        (
            "Limbă",
            "Cuvintele-cheie pot fi scrise în română (cu sau fără diacritice), engleză sau rusă, cu litere mari sau mici.",
        ),
    ],
    about: "Cangurul — mediu de învățare a algoritmilor pentru elevi.",
    shortcuts: "F5 — execută · F6 — pas cu pas · Esc — stop · F7 — verifică · Ctrl+Shift+F — formatează · Ctrl+S/O/N — fișiere",
};

pub static RU: Strings = Strings {
    kw: Keywords {
        step: "ШАГ",
        jump: "ПРЫЖОК",
        rotate: "ПОВОРОТ",
        repeat: "ПОВТОРИ",
        times: "РАЗ",
        end_repeat: "КОНЕЦ ПОВТОРА",
        while_: "ПОКА",
        end_cycle: "КОНЕЦ ЦИКЛА",
        if_: "ЕСЛИ",
        else_: "ИНАЧЕ",
        not: "НЕ",
        border: "КРАЙ",
        line: "ЛИНИЯ",
        procedure: "ПРОЦЕДУРА",
        end_procedure: "КОНЕЦ ПРОЦЕДУРЫ",
        execute: "ВЫПОЛНИТЬ",
        name: "Имя",
    },
    title: "Кенгуренок",
    new: "Новый",
    open: "Открыть…",
    save: "Сохранить",
    save_as: "Сохранить как…",
    examples: "Примеры",
    help: "Помощь",
    settings: "Настройки",
    theme: "Тема",
    theme_system: "Как в системе",
    theme_light: "Светлая",
    theme_dark: "Тёмная",
    run: "Выполнить",
    step_once: "По шагам",
    pause: "Пауза",
    stop: "Стоп",
    reset: "Сброс",
    check: "Проверить",
    format: "Форматировать",
    speed: "Скорость",
    instant: "Мгновенно",
    field: "Поле",
    columns: "Столбцы",
    rows: "Строки",
    show_jumps: "Показывать прыжки",
    show_grid: "Показывать сетку",
    program: "Программа",
    untitled: "без имени",
    ready: "Готов к работе",
    running: "Выполняется…",
    paused: "Пауза",
    finished: "Выполнение завершено",
    stopped: "Выполнение остановлено",
    no_errors: "Программа не содержит ошибок",
    error_line: "Ошибка в строке",
    position: "Позиция",
    direction: "Направление",
    actions: "Выполнено команд",
    lines_drawn: "Линии",
    dirs: ["→ восток", "↓ юг", "← запад", "↑ север"],
    unsaved_title: "Несохранённые изменения",
    unsaved_text: "В программе есть несохранённые изменения. Сохранить?",
    discard: "Не сохранять",
    cancel: "Отмена",
    file_error: "Не удалось открыть/сохранить файл",
    cng_files: "Программы Кенгуренка",
    problems: "Проблемы",
    no_problems: "Проблем не найдено",
    live_check: "Проверять программу во время набора",
    did_you_mean: "Возможно, вы имели в виду",
    fix: "Исправить",
    line_short: "строка",
    summary: |e, w| match (e, w) {
        (0, 0) => "Программа не содержит ошибок".into(),
        (0, w) => format!("Ошибок нет · предупреждений: {w}"),
        (e, 0) => format!("Ошибок: {e}"),
        (e, w) => format!("Ошибок: {e} · предупреждений: {w}"),
    },
    help_text: &[
        (
            "Поле",
            "Кенгуренок движется по узлам сетки. Вначале он стоит в левом верхнем углу и смотрит вправо.",
        ),
        (
            "ШАГ",
            "Кенгуренок делает шаг вперёд и оставляет след (линию).",
        ),
        ("ПРЫЖОК", "Кенгуренок делает шаг вперёд, не оставляя следа."),
        (
            "ПОВОРОТ",
            "Кенгуренок поворачивается на 90° по часовой стрелке.",
        ),
        ("ПОВТОРИ N РАЗ … КОНЕЦ ПОВТОРА", "Повторяет команды N раз."),
        (
            "ПОКА [НЕ] условие … КОНЕЦ ЦИКЛА",
            "Повторяет команды, пока условие истинно.",
        ),
        (
            "ЕСЛИ [НЕ] условие команда1 ИНАЧЕ команда2",
            "Выполняет команду1, если условие истинно, иначе команду2 (ИНАЧЕ можно не писать).",
        ),
        ("КРАЙ", "Истинно, когда кенгуренок смотрит на край поля."),
        (
            "ЛИНИЯ",
            "Истинно, когда впереди уже есть линия (или край поля).",
        ),
        (
            "ПРОЦЕДУРА Имя … КОНЕЦ ПРОЦЕДУРЫ",
            "Описание процедуры. Процедуры пишутся до основной программы.",
        ),
        ("ВЫПОЛНИТЬ Имя", "Выполняет процедуру с данным именем."),
        ("[ … ]", "Основная программа пишется в квадратных скобках."),
        (
            "// комментарий   * комментарий *",
            "Текст после // до конца строки или между двумя * пропускается.",
        ),
        (
            "Проверка",
            "Программа проверяется во время набора: ошибки подчёркнуты красным, предупреждения — жёлтым. В списке «Проблемы» видны все ошибки; щелчок переходит к строке, а «Исправить» заменяет неверное слово.",
        ),
        (
            "Язык",
            "Ключевые слова можно писать по-русски, по-румынски или по-английски, большими или маленькими буквами.",
        ),
    ],
    about: "Кенгуренок — среда для изучения алгоритмов.",
    shortcuts: "F5 — выполнить · F6 — по шагам · Esc — стоп · F7 — проверить · Ctrl+Shift+F — форматировать · Ctrl+S/O/N — файлы",
};

pub static EN: Strings = Strings {
    kw: Keywords {
        step: "STEP",
        jump: "JUMP",
        rotate: "ROTATE",
        repeat: "REPEAT",
        times: "TIMES",
        end_repeat: "END REPEAT",
        while_: "WHILE",
        end_cycle: "END CYCLE",
        if_: "IF",
        else_: "ELSE",
        not: "NOT",
        border: "IS_BORDER",
        line: "IS_LINE",
        procedure: "PROCEDURE",
        end_procedure: "END PROCEDURE",
        execute: "EXECUTE",
        name: "Name",
    },
    title: "Kangaroo",
    new: "New",
    open: "Open…",
    save: "Save",
    save_as: "Save as…",
    examples: "Examples",
    help: "Help",
    settings: "Settings",
    theme: "Theme",
    theme_system: "System",
    theme_light: "Light",
    theme_dark: "Dark",
    run: "Run",
    step_once: "Step by step",
    pause: "Pause",
    stop: "Stop",
    reset: "Reset",
    check: "Check",
    format: "Format",
    speed: "Speed",
    instant: "Instant",
    field: "Field",
    columns: "Columns",
    rows: "Rows",
    show_jumps: "Show jumps",
    show_grid: "Show grid",
    program: "Program",
    untitled: "untitled",
    ready: "Ready",
    running: "Running…",
    paused: "Paused",
    finished: "Execution complete",
    stopped: "Execution stopped",
    no_errors: "The program has no errors",
    error_line: "Error on line",
    position: "Position",
    direction: "Direction",
    actions: "Commands executed",
    lines_drawn: "Lines",
    dirs: ["→ east", "↓ south", "← west", "↑ north"],
    unsaved_title: "Unsaved changes",
    unsaved_text: "The program has unsaved changes. Save them?",
    discard: "Don't save",
    cancel: "Cancel",
    file_error: "Could not open/save the file",
    cng_files: "Kangaroo programs",
    problems: "Problems",
    no_problems: "No problems found",
    live_check: "Check the program while I type",
    did_you_mean: "Did you mean",
    fix: "Fix",
    line_short: "line",
    summary: |e, w| match (e, w) {
        (0, 0) => "The program has no errors".into(),
        (0, w) => format!("No errors · warnings: {w}"),
        (e, 0) => format!("Errors: {e}"),
        (e, w) => format!("Errors: {e} · warnings: {w}"),
    },
    help_text: &[
        (
            "Field",
            "The kangaroo moves on the nodes of a grid. It starts in the top-left corner, facing right.",
        ),
        (
            "STEP",
            "The kangaroo moves one step forward and leaves a trace (a line).",
        ),
        (
            "JUMP",
            "The kangaroo moves one step forward without leaving a trace.",
        ),
        ("ROTATE", "The kangaroo turns 90° clockwise."),
        (
            "REPEAT N TIMES … END REPEAT",
            "Repeats the commands N times.",
        ),
        (
            "WHILE [NOT] condition … END CYCLE",
            "Repeats the commands while the condition is true.",
        ),
        (
            "IF [NOT] condition command1 ELSE command2",
            "Runs command1 if the condition is true, otherwise command2 (ELSE is optional).",
        ),
        (
            "IS_BORDER",
            "True when the kangaroo faces the edge of the field.",
        ),
        (
            "IS_LINE",
            "True when there is already a line (or the edge) in front of the kangaroo.",
        ),
        (
            "PROCEDURE Name … END PROCEDURE",
            "Defines a procedure. Procedures are written before the main program.",
        ),
        ("EXECUTE Name", "Runs the procedure with the given name."),
        (
            "[ … ]",
            "The main program is written between square brackets.",
        ),
        (
            "// comment   * comment *",
            "Text after // to the end of the line, or between two *, is ignored.",
        ),
        (
            "Checking",
            "The program is checked while you type: errors are underlined in red, warnings in yellow. The \"Problems\" list shows every mistake; click one to jump to its line, and \"Fix\" replaces the misspelled word.",
        ),
        (
            "Language",
            "Keywords may be written in English, Romanian or Russian, in upper or lower case.",
        ),
    ],
    about: "Kangaroo — a learning environment for algorithms.",
    shortcuts: "F5 — run · F6 — step · Esc — stop · F7 — check · Ctrl+Shift+F — format · Ctrl+S/O/N — files",
};
