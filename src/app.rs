//! Interfața grafică.

use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::text::{CCursor, CCursorRange, LayoutJob};
use egui::{
    Align, Align2, Color32, CornerRadius, FontFamily, FontId, Key, KeyboardShortcut, Layout,
    Margin, Modifiers, Pos2, Rect, RichText, Sense, Shape, Stroke, TextBuffer, TextFormat,
    TextureHandle, Theme, ThemePreference, Ui, Vec2, ViewportCommand, pos2, vec2,
};

use crate::i18n::{Keywords, Lang, Strings};
use crate::lang::{self, Action, CompileError, TokenClass};
use crate::vm::{RuntimeError, Tick, Vm};
use crate::world::{Dir, Move, World};

pub const EXAMPLES: &[(&str, &str)] = &[
    ("Cangur", include_str!("../examples/Cangur.cng")),
    ("Litera L", include_str!("../examples/Litera_L.cng")),
    ("Spirala (RO)", include_str!("../examples/Spirala(Rom).cng")),
    ("Spiral (EN)", include_str!("../examples/Spirala(Eng).cng")),
    ("Спираль (RU)", include_str!("../examples/Spirala(Rus).cng")),
    ("Pătrat", include_str!("../examples/Patrat.cng")),
    ("Figura 1", include_str!("../examples/figura1.cng")),
    ("Figura 2", include_str!("../examples/figura2.cng")),
    ("Figura 3", include_str!("../examples/figura3.cng")),
];

const DEFAULT_PROGRAM: &str = "// Primul meu program\n[\nREPETĂ 4 ORI\n    PAS\n    PAS\n    ROTIRE\nSFÎRȘITUL REPETĂRII\n]\n";

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThemeChoice {
    System,
    Light,
    Dark,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Settings {
    pub lang: Lang,
    pub theme: ThemeChoice,
    pub speed: f32,
    pub instant: bool,
    pub cols: i32,
    pub rows: i32,
    pub show_grid: bool,
    pub show_jumps: bool,
    /// Analiza programului în timp ce elevul scrie.
    pub live_check: bool,
    pub code: String,
    pub path: Option<PathBuf>,
    pub dirty: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            lang: Lang::Ro,
            theme: ThemeChoice::System,
            speed: 5.0,
            instant: false,
            cols: 20,
            rows: 20,
            show_grid: true,
            show_jumps: false,
            live_check: true,
            code: DEFAULT_PROGRAM.into(),
            path: None,
            dirty: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Idle,
    Running,
    Paused,
}

#[derive(Clone)]
enum Status {
    Ready,
    Running,
    Paused,
    Finished,
    Stopped,
    /// Rezultatul butonului „Verifică”: (erori, avertismente).
    Checked(usize, usize),
    /// Prima eroare și numărul total de erori.
    Compile(CompileError, usize),
    Runtime(RuntimeError, usize),
    File(String),
}

struct Anim {
    mv: Move,
    start: f64,
    dur: f64,
    /// Linia desenată de acest pas este nouă (deci se desenează treptat).
    new_line: bool,
}

#[derive(Clone, Copy)]
enum Pending {
    New,
    Open,
    Example(usize),
    Quit,
}

/// O imagine în trei mărimi (1, 1/2, 1/4). Filtrarea liniară arată bine doar până la
/// o micșorare de ~2 ori, iar mipmap-urile cer OpenGL 3, deci micșorările se fac aici.
struct Sprite([TextureHandle; 3]);

impl Sprite {
    fn load(ctx: &egui::Context, name: &str, bytes: &[u8]) -> Sprite {
        let img = image::load_from_memory(bytes)
            .expect("imagine invalidă")
            .to_rgba8();
        let level = |i: usize| {
            let side = img.width() >> i;
            let img =
                image::imageops::resize(&img, side, side, image::imageops::FilterType::Lanczos3);
            let size = [side as usize, side as usize];
            let pixels = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
            ctx.load_texture(
                format!("{name}-{side}"),
                pixels,
                egui::TextureOptions::LINEAR,
            )
        };
        Sprite([level(0), level(1), level(2)])
    }

    /// Cea mai mică textură care are cel puțin `px` pixeli fizici.
    fn at(&self, px: f32) -> &TextureHandle {
        self.0
            .iter()
            .rev()
            .find(|t| t.size()[0] as f32 >= px)
            .unwrap_or(&self.0[0])
    }
}

struct Sprites {
    east: Sprite,
    south: Sprite,
    west: Sprite,
    north: Sprite,
    tired: Sprite,
    logo: Sprite,
}

/// Colțul de sus-stânga al câmpului și latura unei celule în zona `area`.
/// Sus rămâne loc pentru cangurul din rândul 0: stă deasupra nodului și, la
/// mărimea maximă, ar ieși din zonă (urechile ar fi tăiate).
fn field_layout(area: Rect, cols: f32, rows: f32) -> (Pos2, f32) {
    let pad: f32 = 40.0;
    let pad_top = pad.max(sprite_size(f32::MAX) * 0.88);
    let cell = ((area.width() - 2.0 * pad) / cols)
        .min((area.height() - pad - pad_top) / rows)
        .max(4.0);
    let size = vec2(cell * cols, cell * rows);
    let free_y = area.height() - pad - pad_top - size.y;
    let origin = pos2(
        area.center().x - size.x / 2.0,
        area.top() + pad_top + free_y / 2.0,
    );
    (origin, cell)
}

/// Mărimea cangurului pe câmp, după latura celulei.
fn sprite_size(cell: f32) -> f32 {
    (cell * 1.6).clamp(28.0, 72.0)
}

impl Sprites {
    fn load(ctx: &egui::Context) -> Sprites {
        Sprites {
            east: Sprite::load(ctx, "east", include_bytes!("../assets/sprites/east.png")),
            south: Sprite::load(ctx, "south", include_bytes!("../assets/sprites/south.png")),
            west: Sprite::load(ctx, "west", include_bytes!("../assets/sprites/west.png")),
            north: Sprite::load(ctx, "north", include_bytes!("../assets/sprites/north.png")),
            tired: Sprite::load(ctx, "tired", include_bytes!("../assets/sprites/tired.png")),
            logo: Sprite::load(ctx, "logo", include_bytes!("../assets/logo.png")),
        }
    }

    fn for_dir(&self, d: Dir) -> &Sprite {
        match d {
            Dir::East => &self.east,
            Dir::South => &self.south,
            Dir::West => &self.west,
            Dir::North => &self.north,
        }
    }
}

pub fn decode_png(bytes: &[u8]) -> egui::ColorImage {
    let img = image::load_from_memory(bytes)
        .expect("imagine invalidă")
        .to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw())
}

/// Culorile interfeței pentru tema curentă.
struct Pal {
    accent: Color32,
    ink: Color32,
    grid: Color32,
    border: Color32,
    field_bg: Color32,
    muted: Color32,
    ok: Color32,
    err: Color32,
    warn: Color32,
    current_line: Color32,
    error_line: Color32,
    action: Color32,
    control: Color32,
    cond: Color32,
    number: Color32,
    bracket: Color32,
    comment: Color32,
    text: Color32,
}

impl Pal {
    fn new(dark: bool) -> Pal {
        if dark {
            Pal {
                accent: Color32::from_rgb(0xF0, 0x8A, 0x3C),
                ink: Color32::from_rgb(0x6C, 0xB4, 0xFF),
                grid: Color32::from_rgb(0x34, 0x39, 0x44),
                border: Color32::from_rgb(0x5A, 0x62, 0x72),
                field_bg: Color32::from_rgb(0x20, 0x24, 0x2C),
                muted: Color32::from_rgb(0x8A, 0x92, 0xA0),
                ok: Color32::from_rgb(0x5B, 0xD1, 0x8C),
                err: Color32::from_rgb(0xFF, 0x6B, 0x6B),
                warn: Color32::from_rgb(0xE8, 0xB0, 0x4A),
                current_line: Color32::from_rgba_unmultiplied(0xF0, 0x8A, 0x3C, 0x38),
                error_line: Color32::from_rgba_unmultiplied(0xFF, 0x50, 0x50, 0x40),
                action: Color32::from_rgb(0x6C, 0xB4, 0xFF),
                control: Color32::from_rgb(0xF0, 0x8A, 0x3C),
                cond: Color32::from_rgb(0xC5, 0x92, 0xFF),
                number: Color32::from_rgb(0x5B, 0xD1, 0x8C),
                bracket: Color32::from_rgb(0xE8, 0xC5, 0x4A),
                comment: Color32::from_rgb(0x74, 0x7C, 0x8A),
                text: Color32::from_rgb(0xDD, 0xE1, 0xE8),
            }
        } else {
            Pal {
                accent: Color32::from_rgb(0xE0, 0x6C, 0x1E),
                ink: Color32::from_rgb(0x1F, 0x5F, 0xD6),
                grid: Color32::from_rgb(0xE3, 0xE0, 0xD8),
                border: Color32::from_rgb(0xA8, 0xA2, 0x96),
                field_bg: Color32::from_rgb(0xFF, 0xFD, 0xF8),
                muted: Color32::from_rgb(0x6E, 0x6A, 0x62),
                ok: Color32::from_rgb(0x1E, 0x8E, 0x4E),
                err: Color32::from_rgb(0xD0, 0x2F, 0x2F),
                warn: Color32::from_rgb(0xB4, 0x6A, 0x00),
                current_line: Color32::from_rgba_unmultiplied(0xF5, 0x9E, 0x2B, 0x40),
                error_line: Color32::from_rgba_unmultiplied(0xE0, 0x30, 0x30, 0x30),
                action: Color32::from_rgb(0x1F, 0x5F, 0xD6),
                control: Color32::from_rgb(0xC2, 0x55, 0x0C),
                cond: Color32::from_rgb(0x7C, 0x3A, 0xED),
                number: Color32::from_rgb(0x15, 0x80, 0x3D),
                bracket: Color32::from_rgb(0xA1, 0x62, 0x07),
                comment: Color32::from_rgb(0x8C, 0x88, 0x80),
                text: Color32::from_rgb(0x22, 0x24, 0x28),
            }
        }
    }

    fn token(&self, c: TokenClass) -> Color32 {
        match c {
            TokenClass::Plain => self.text,
            TokenClass::Action => self.action,
            TokenClass::Control => self.control,
            TokenClass::Condition => self.cond,
            TokenClass::Number => self.number,
            TokenClass::Bracket => self.bracket,
            TokenClass::Comment => self.comment,
        }
    }
}

pub struct CangurApp {
    s: Settings,
    world: World,
    vm: Option<Vm>,
    mode: Mode,
    anim: Option<Anim>,
    line: Option<usize>,
    error_line: Option<usize>,
    status: Status,
    crashed: bool,
    sprites: Sprites,
    pending: Option<Pending>,
    show_help: bool,
    /// Selecția din editor (în caractere; goală = doar cursorul);
    /// `None` până la primul click.
    cursor: Option<Range<usize>>,
    allow_close: bool,
    title: String,
    analysis: lang::Analysis,
    /// Textul pentru care e valabilă `analysis` (`None` = neanalizat).
    analyzed: Option<String>,
    /// Porțiunea de text la care trebuie dus cursorul (click în lista de probleme).
    jump_to: Option<Range<usize>>,
    /// Editorul primește focusul după ce e desenat (click-ul pe un buton
    /// i l-ar lua în același cadru).
    focus_editor: bool,
    /// Variantele propuse în timp ce elevul scrie.
    completion: Option<Completion>,
}

/// O comandă de inserat: textul dinaintea exemplului, exemplul (rămâne
/// selectat, ca următoarea comandă să-l înlocuiască) și textul de după.
#[derive(Clone, Default)]
struct Snippet {
    before: String,
    demo: String,
    after: String,
}

impl Snippet {
    fn word(word: &str) -> Snippet {
        Snippet {
            before: word.into(),
            ..Snippet::default()
        }
    }

    /// Un bloc cu `step` ca exemplu în interior.
    fn block(head: &str, step: &str, end: &str) -> Snippet {
        Snippet {
            before: format!("{head}\n    "),
            demo: step.into(),
            after: format!("\n{end}"),
        }
    }

    /// DACĂ pe mai multe rânduri; comanda de după condiție e selectată.
    fn if_else(kw: &Keywords, cond: &str) -> Snippet {
        Snippet {
            before: format!("{} {cond}\n    ", kw.if_),
            demo: kw.rotate.into(),
            after: format!("\n{}\n    {}", kw.else_, kw.step),
        }
    }

    fn text(&self) -> String {
        [&self.before, &self.demo, &self.after]
            .map(String::as_str)
            .concat()
    }

    /// Primul rând, cu „…” dacă blocul continuă.
    fn label(&self) -> String {
        let text = self.text();
        match text.split_once('\n') {
            Some((first, _)) => format!("{first} …"),
            None => text,
        }
    }
}

/// Lista de variante deschisă sub cursor.
struct Completion {
    /// Comanda scrisă pe rând (în caractere), înlocuită de varianta aleasă.
    range: Range<usize>,
    items: Vec<Snippet>,
    selected: usize,
    /// Unde a fost desenată lista (pentru click-urile cu mouse-ul).
    rect: Option<Rect>,
}

/// O problemă de afișat în editor și în lista de probleme.
struct Diag {
    line: usize,
    span: Range<usize>,
    error: bool,
    message: String,
    hint: Option<String>,
}

const STORAGE_KEY: &str = "cangurul";
const EDITOR_ID: &str = "editor";

impl CangurApp {
    pub fn new(cc: &eframe::CreationContext<'_>, open: Option<PathBuf>) -> CangurApp {
        setup_style(&cc.egui_ctx);
        let s: Settings = cc
            .storage
            .and_then(|st| eframe::get_value(st, STORAGE_KEY))
            .unwrap_or_default();
        let mut app = CangurApp {
            world: World::new(s.cols, s.rows),
            s,
            vm: None,
            mode: Mode::Idle,
            anim: None,
            line: None,
            error_line: None,
            status: Status::Ready,
            crashed: false,
            sprites: Sprites::load(&cc.egui_ctx),
            pending: None,
            show_help: false,
            cursor: None,
            allow_close: false,
            title: String::new(),
            analysis: lang::Analysis::default(),
            analyzed: None,
            jump_to: None,
            focus_editor: false,
            completion: None,
        };
        app.apply_theme(&cc.egui_ctx);
        if let Some(p) = open {
            app.load_file(&p);
        }
        app
    }

    fn t(&self) -> &'static Strings {
        self.s.lang.t()
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        ctx.set_theme(match self.s.theme {
            ThemeChoice::System => ThemePreference::System,
            ThemeChoice::Light => ThemePreference::Light,
            ThemeChoice::Dark => ThemePreference::Dark,
        });
    }

    // ------------------------------------------------------- execuție ---

    // -------------------------------------------------------- analiză ---

    fn analyze_now(&mut self) {
        if self.analyzed.as_ref() != Some(&self.s.code) {
            self.analysis = lang::analyze(&self.s.code);
            self.analyzed = Some(self.s.code.clone());
        }
    }

    /// Ține analiza la zi cu textul (sau o șterge dacă verificarea automată e oprită).
    fn refresh_analysis(&mut self) {
        if self.analyzed.as_ref() == Some(&self.s.code) {
            return;
        }
        if self.s.live_check {
            self.analyze_now();
        } else {
            self.analysis = lang::Analysis::default();
            self.analyzed = None;
        }
    }

    fn diagnostics(&self) -> Vec<Diag> {
        let t = self.t();
        let lang = self.s.lang;
        let errors = self.analysis.errors.iter().map(|e| Diag {
            line: e.line,
            span: e.span.clone(),
            error: true,
            message: t.compile_error(lang, &e.kind),
            hint: e.hint.clone(),
        });
        let warnings = self.analysis.warnings.iter().map(|w| Diag {
            line: w.line,
            span: w.span.clone(),
            error: false,
            message: t.warning(lang, &w.kind),
            hint: None,
        });
        let mut all: Vec<Diag> = errors.chain(warnings).collect();
        all.sort_by_key(|d| (d.line, d.span.start));
        all
    }

    /// Înlocuiește cuvântul greșit cu sugestia.
    fn apply_fix(&mut self, span: Range<usize>, hint: &str) {
        if self.s.code.get(span.clone()).is_some() {
            self.s.code.replace_range(span.clone(), hint);
            self.s.dirty = true;
            self.status = Status::Ready;
            self.jump_to = Some(span.start..span.start + hint.len());
        }
    }

    // ------------------------------------------------------- execuție ---

    fn compile(&mut self) -> Option<lang::Program> {
        self.analyze_now();
        match &self.analysis.program {
            Some(p) => Some(p.clone()),
            None => {
                let first = self.analysis.errors[0].clone();
                self.status = Status::Compile(first, self.analysis.errors.len());
                None
            }
        }
    }

    fn start(&mut self, mode: Mode) -> bool {
        self.error_line = None;
        let Some(p) = self.compile() else {
            return false;
        };
        self.world = World::new(self.s.cols, self.s.rows);
        self.vm = Some(Vm::new(p));
        self.mode = mode;
        self.anim = None;
        self.crashed = false;
        self.line = None;
        self.status = if mode == Mode::Running {
            Status::Running
        } else {
            Status::Paused
        };
        true
    }

    fn run(&mut self) {
        match self.mode {
            Mode::Idle => {
                self.start(Mode::Running);
            }
            Mode::Paused => {
                self.mode = Mode::Running;
                self.status = Status::Running;
            }
            Mode::Running => {}
        }
    }

    fn pause(&mut self) {
        if self.mode == Mode::Running {
            self.mode = Mode::Paused;
            self.status = Status::Paused;
        }
    }

    fn step_once(&mut self, now: f64) {
        match self.mode {
            Mode::Idle => {
                if self.start(Mode::Paused) {
                    self.advance(now, true);
                }
            }
            Mode::Running => self.pause(),
            Mode::Paused => {
                if self.anim.is_none() {
                    self.advance(now, true);
                }
            }
        }
    }

    fn stop(&mut self) {
        if self.mode != Mode::Idle {
            self.finish(Status::Stopped);
        }
    }

    fn reset(&mut self) {
        self.vm = None;
        self.mode = Mode::Idle;
        self.anim = None;
        self.line = None;
        self.error_line = None;
        self.crashed = false;
        self.world = World::new(self.s.cols, self.s.rows);
        self.status = Status::Ready;
    }

    fn check(&mut self) {
        self.error_line = None;
        self.analyze_now();
        self.status = Status::Checked(self.analysis.errors.len(), self.analysis.warnings.len());
    }

    fn finish(&mut self, status: Status) {
        self.vm = None;
        self.mode = Mode::Idle;
        self.anim = None;
        self.line = None;
        self.status = status;
    }

    fn step_duration(&self) -> f64 {
        0.9 * 0.68f64.powf(self.s.speed as f64 - 1.0)
    }

    /// Execută până la următoarea acțiune a cangurului.
    fn advance(&mut self, now: f64, animate: bool) {
        let Some(vm) = &mut self.vm else { return };
        let lines_before = self.world.lines.len();
        match vm.tick(&mut self.world, 50_000) {
            Tick::Moved(mv, line) => {
                self.line = Some(line);
                if animate {
                    self.anim = Some(Anim {
                        mv,
                        start: now,
                        dur: self.step_duration(),
                        new_line: self.world.lines.len() > lines_before,
                    });
                }
            }
            Tick::Busy => {}
            Tick::Finished => self.finish(Status::Finished),
            Tick::Failed(e, line) => {
                self.error_line = Some(line);
                self.crashed = e == RuntimeError::OutOfField;
                self.finish(Status::Runtime(e, line));
            }
        }
    }

    fn drive(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        if self.anim.as_ref().is_some_and(|a| now >= a.start + a.dur) {
            self.anim = None;
        }
        if self.mode == Mode::Running && self.anim.is_none() {
            if self.s.instant {
                let deadline = std::time::Instant::now() + std::time::Duration::from_millis(12);
                while self.mode == Mode::Running && std::time::Instant::now() < deadline {
                    for _ in 0..200 {
                        self.advance(now, false);
                        if self.mode != Mode::Running {
                            break;
                        }
                    }
                }
            } else {
                self.advance(now, true);
            }
        }
        if self.mode == Mode::Running || self.anim.is_some() {
            ctx.request_repaint();
        }
    }

    // --------------------------------------------------------- fișiere ---

    fn file_name(&self) -> String {
        self.s
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .map_or_else(
                || self.t().untitled.to_string(),
                |n| n.to_string_lossy().into_owned(),
            )
    }

    fn load_file(&mut self, path: &Path) {
        match std::fs::read(path) {
            Ok(bytes) => {
                self.s.code = decode_cng(&bytes);
                self.cursor = None;
                self.s.path = Some(path.to_path_buf());
                self.s.dirty = false;
                self.reset();
            }
            Err(e) => self.status = Status::File(format!("{}: {e}", self.t().file_error)),
        }
    }

    fn save(&mut self, save_as: bool) -> bool {
        let path = match (&self.s.path, save_as) {
            (Some(p), false) => Some(p.clone()),
            _ => rfd::FileDialog::new()
                .add_filter(self.t().cng_files, &["cng"])
                .set_file_name(format!("{}.cng", self.file_name().trim_end_matches(".cng")))
                .save_file(),
        };
        let Some(mut path) = path else { return false };
        if path.extension().is_none() {
            path.set_extension("cng");
        }
        match std::fs::write(&path, self.s.code.replace('\n', "\r\n")) {
            Ok(()) => {
                self.s.path = Some(path);
                self.s.dirty = false;
                true
            }
            Err(e) => {
                self.status = Status::File(format!("{}: {e}", self.t().file_error));
                false
            }
        }
    }

    fn request(&mut self, p: Pending, ctx: &egui::Context) {
        if self.s.dirty {
            self.pending = Some(p);
        } else {
            self.perform(p, ctx);
        }
    }

    fn perform(&mut self, p: Pending, ctx: &egui::Context) {
        match p {
            Pending::New => {
                self.s.code = "[\n\n]\n".into();
                self.s.path = None;
                self.s.dirty = false;
                self.cursor = Some(2..2);
                self.reset();
            }
            Pending::Open => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter(self.t().cng_files, &["cng", "txt"])
                    .pick_file()
                {
                    self.load_file(&path);
                }
            }
            Pending::Example(i) => {
                self.s.code = EXAMPLES[i].1.to_string();
                self.cursor = None;
                self.s.path = None;
                self.s.dirty = false;
                self.reset();
            }
            Pending::Quit => {
                self.allow_close = true;
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        }
    }

    /// Formatează programul; cursorul rămâne lângă același cuvânt, iar
    /// schimbarea se anulează cu un singur Ctrl+Z.
    fn format_code(&mut self, ctx: &egui::Context) {
        let new = lang::format_program(&self.s.code);
        if new != self.s.code {
            let sel = self.cursor.clone().unwrap_or_default();
            let map = |at| respaced_cursor(&self.s.code, &new, at);
            let cursor = map(sel.start)..map(sel.end);
            let id = egui::Id::unique(EDITOR_ID);
            let mut state = egui::text_edit::TextEditState::load(ctx, id).unwrap_or_default();
            // Două puncte de anulare: textul vechi și cel formatat.
            let mut undoer = state.undoer();
            let old_range = state.cursor.char_range().unwrap_or_default();
            undoer.add_undo(&(old_range, self.s.code.clone()));
            let range = CCursorRange::two(CCursor::new(cursor.start), CCursor::new(cursor.end));
            undoer.add_undo(&(range, new.clone()));
            state.set_undoer(undoer);
            state.cursor.set_char_range(Some(range));
            state.store(ctx, id);
            self.s.code = new;
            self.s.dirty = true;
            self.cursor = Some(cursor);
        }
        self.focus_editor = true;
    }

    /// Scrie comanda `before` + `demo` + `after` în program. Selecția e
    /// înlocuită de comandă; fără selecție, comanda ocupă rândul gol al
    /// cursorului sau un rând nou, deasupra ori dedesubtul lui. Rândurile noi
    /// primesc indentarea blocului în care ajung, iar exemplul din interiorul
    /// comenzii (`demo`) rămâne selectat, ca următoarea comandă să-l înlocuiască.
    /// După o comandă fără exemplu, cursorul trece pe un rând nou.
    fn insert_snippet(&mut self, snippet: &Snippet) {
        let Snippet {
            before,
            demo,
            after,
        } = snippet;
        let (before, demo, after) = (before.as_str(), demo.as_str(), after.as_str());
        let code = &self.s.code;
        let byte = |c: usize| code.char_indices().nth(c).map_or(code.len(), |(b, _)| b);
        // Fără cursor, comanda se adaugă la sfârșitul programului principal.
        let default = code.rfind(']').unwrap_or(code.len());
        let sel = match &self.cursor {
            Some(r) => byte(r.start)..byte(r.end),
            None => default..default,
        };
        let line_start = code[..sel.start].rfind('\n').map_or(0, |i| i + 1);
        let line_end = code[sel.start..]
            .find('\n')
            .map_or(code.len(), |i| sel.start + i);
        let (range, prefix, suffix) = if !sel.is_empty() {
            (sel, "", "")
        } else if code[line_start..line_end].trim().is_empty() {
            (line_start..line_end, "", "")
        } else if code[line_start..sel.start].trim().is_empty() {
            (line_start..line_start, "", "\n")
        } else {
            (line_end..line_end, "\n", "")
        };
        // Rândul nou primește un cuvânt provizoriu, ca formatarea să-i dea
        // indentarea; cuvântul e scos apoi.
        let rest = &code[range.end..];
        let mut lines = rest.split('\n');
        let line_rest_blank = lines.next().unwrap_or("").trim().is_empty();
        // Comanda a înlocuit-o pe cea de după DACĂ: urmează ALTFEL și comanda
        // lui, care e selectată în loc să se facă un rând nou.
        let else_follows = demo.is_empty()
            && line_rest_blank
            && lines.next().is_some_and(|l| lang::is_else(l.trim()))
            && lines.next().is_some_and(|l| {
                let l = l.trim();
                !l.is_empty() && !l.contains(char::is_whitespace)
            });
        let new_line = demo.is_empty() && line_rest_blank && !else_follows;
        let placeholder = if new_line { "\n_" } else { "" };

        let new = [
            &code[..range.start],
            prefix,
            before,
            demo,
            after,
            placeholder,
            suffix,
            rest,
        ]
        .concat();
        // Indentarea rândurilor atinse vine din formatare; restul rămâne.
        let first = code[..range.start].matches('\n').count() + prefix.len();
        let last = first
            + [before, demo, after, placeholder]
                .concat()
                .matches('\n')
                .count();
        let code = reindent_lines(&new, first..=last);

        let start = new[..range.start].chars().count() + prefix.len() + before.chars().count();
        let end = start + demo.chars().count();
        let (a, b) = if demo.is_empty() {
            let at = end + after.chars().count() + placeholder.len().saturating_sub(1);
            (at, at)
        } else {
            (start, end)
        };
        let mut cursor = reindented_cursor(&new, &code, a)..reindented_cursor(&new, &code, b);
        if else_follows {
            let at = code
                .char_indices()
                .nth(cursor.end)
                .map_or(code.len(), |(b, _)| b);
            let line = code[at..].split('\n').nth(2).unwrap_or("");
            let line_start: usize = code[at..]
                .split('\n')
                .take(2)
                .map(|l| l.chars().count() + 1)
                .sum::<usize>()
                + cursor.end;
            let indent = line.chars().take_while(|c| c.is_whitespace()).count();
            let start = line_start + indent;
            cursor = start..start + line.trim().chars().count();
        }
        let mut code = code;
        if new_line {
            let at = code
                .char_indices()
                .nth(cursor.start)
                .map_or(code.len(), |(b, _)| b);
            code.remove(at);
        }
        self.s.code = code;
        self.s.dirty = true;
        self.cursor = Some(cursor);
        self.focus_editor = true;
    }

    /// Enter: textul de după cursor trece pe un rând nou, cu indentarea
    /// blocului; rândul părăsit e reindentat și el (SFÎRȘITUL, ALTFEL se
    /// aliniază cu începutul blocului). Întoarce noua poziție a cursorului.
    fn new_line(&mut self, sel: Range<usize>) -> usize {
        let code = &self.s.code;
        let byte = |c: usize| code.char_indices().nth(c).map_or(code.len(), |(b, _)| b);
        let (a, b) = (byte(sel.start), byte(sel.end));
        let rest = code[b..].trim_start_matches([' ', '\t']);
        // Pe un rând gol, cuvântul provizoriu îi dă indentarea la formatare.
        let placeholder = if rest.split('\n').next().unwrap_or("").trim().is_empty() {
            "_"
        } else {
            ""
        };
        let new = [&code[..a], "\n", placeholder, rest].concat();
        let line = code[..a].matches('\n').count();
        let mut out = reindent_lines(&new, line..=line + 1);
        let at = reindented_cursor(&new, &out, code[..a].chars().count() + 1);
        if !placeholder.is_empty() {
            let b = out.char_indices().nth(at).map_or(out.len(), |(b, _)| b);
            out.remove(b);
        }
        self.s.code = out;
        self.cursor = Some(at..at);
        at
    }

    /// Variantele care continuă comanda `typed` scrisă la începutul rândului;
    /// `before` e textul programului de dinaintea ei.
    fn suggestions(&self, typed: &str, before: &str) -> Vec<Snippet> {
        let key = |s: &str| {
            lang::normalize(s)
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        };
        let typed_key = key(typed);
        if !typed_key.chars().any(char::is_alphanumeric) {
            return Vec::new();
        }
        let kw = &self.t().kw;
        let n = typed
            .split_whitespace()
            .nth(1)
            .filter(|a| a.chars().all(|c| c.is_ascii_digit()))
            .unwrap_or("4");
        let not = |c: &str| format!("{} {c}", kw.not);
        let mut all = vec![
            Snippet::word(kw.step),
            Snippet::word(kw.jump),
            Snippet::word(kw.rotate),
            Snippet::block(
                &format!("{} {n} {}", kw.repeat, kw.times),
                kw.step,
                kw.end_repeat,
            ),
        ];
        for c in [
            not(kw.border),
            kw.border.into(),
            not(kw.line),
            kw.line.into(),
        ] {
            all.push(Snippet::block(
                &format!("{} {c}", kw.while_),
                kw.step,
                kw.end_cycle,
            ));
        }
        for c in [
            kw.border.into(),
            not(kw.border),
            kw.line.into(),
            not(kw.line),
        ] {
            all.push(Snippet::if_else(kw, &c));
        }
        all.push(Snippet::word(kw.else_));
        all.push(Snippet::block(
            &format!("{} {}", kw.procedure, kw.name),
            kw.step,
            kw.end_procedure,
        ));
        let procs = lang::procedure_names(&self.s.code);
        if procs.is_empty() {
            all.push(Snippet {
                before: format!("{} ", kw.execute),
                demo: kw.name.into(),
                after: String::new(),
            });
        }
        for p in procs {
            all.push(Snippet::word(&format!("{} {p}", kw.execute)));
        }
        // Sfârșitul blocului deschis e propus primul.
        let mut ends = [
            (lang::BlockKind::Repeat, kw.end_repeat),
            (lang::BlockKind::Cycle, kw.end_cycle),
            (lang::BlockKind::Procedure, kw.end_procedure),
        ];
        let open = lang::open_block(before);
        ends.sort_by_key(|(kind, _)| Some(*kind) != open);
        all.extend(ends.map(|(_, end)| Snippet::word(end)));

        // Rămân variantele care adaugă ceva în afara exemplului: după
        // „DACĂ E_MARGINE” Enter face rând nou, nu alege DACĂ pe un rând.
        all.retain(|s| {
            let first = key(s.text().split('\n').next().unwrap_or(""));
            first.starts_with(&typed_key) && first != typed_key && key(&s.before) != typed_key
        });
        all
    }

    /// Deschide lista de variante pentru comanda scrisă înaintea cursorului.
    fn update_completion(&mut self) {
        self.completion = None;
        let Some(cursor) = self.cursor.clone().filter(|r| r.is_empty()) else {
            return;
        };
        let code = &self.s.code;
        let end = code
            .char_indices()
            .nth(cursor.end)
            .map_or(code.len(), |(b, _)| b);
        let line_start = code[..end].rfind('\n').map_or(0, |i| i + 1);
        let typed = code[line_start..end].trim_start();
        let start = end - typed.len();
        let items = self.suggestions(typed, &code[..start]);
        if !items.is_empty() {
            self.completion = Some(Completion {
                range: code[..start].chars().count()..cursor.end,
                items,
                selected: 0,
                rect: None,
            });
        }
    }

    fn accept_completion(&mut self, i: usize) {
        if let Some(c) = self.completion.take() {
            self.cursor = Some(c.range);
            self.insert_snippet(&c.items[i]);
        }
    }

    // ------------------------------------------------------------- UI ---

    fn shortcuts(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        let (f5, f6, f7, esc) = ctx.input(|i| {
            (
                i.key_pressed(Key::F5),
                i.key_pressed(Key::F6),
                i.key_pressed(Key::F7),
                i.key_pressed(Key::Escape),
            )
        });
        let shortcut = |k| {
            ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, k)))
        };
        if f5 {
            if self.mode == Mode::Running {
                self.pause()
            } else {
                self.run()
            }
        }
        if f6 {
            self.step_once(now);
        }
        if f7 {
            self.check();
        }
        if esc {
            self.stop();
        }
        if shortcut(Key::S) {
            self.save(false);
        }
        if shortcut(Key::O) {
            self.request(Pending::Open, ctx);
        }
        if shortcut(Key::N) {
            self.request(Pending::New, ctx);
        }
        let format = KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, Key::F);
        if ctx.input_mut(|i| i.consume_shortcut(&format)) && self.mode == Mode::Idle {
            self.format_code(ctx);
        }

        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_path_buf())
                .collect()
        });
        if let Some(p) = dropped.into_iter().next()
            && !self.s.dirty
        {
            self.load_file(&p);
        }

        if ctx.input(|i| i.viewport().close_requested()) && self.s.dirty && !self.allow_close {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.pending = Some(Pending::Quit);
        }

        let title = format!(
            "{} — {}{}",
            self.t().title,
            self.file_name(),
            if self.s.dirty { " •" } else { "" }
        );
        if title != self.title {
            ctx.send_viewport_cmd(ViewportCommand::Title(title.clone()));
            self.title = title;
        }
    }

    fn top_bar(&mut self, ui: &mut Ui) {
        let t = self.t();
        let ctx = ui.ctx().clone();
        egui::Panel::top("top")
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().panel_fill)
                    .inner_margin(Margin::symmetric(12, 8)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Image::new(self.sprites.logo.at(30.0 * ui.pixels_per_point()))
                            .fit_to_exact_size(vec2(30.0, 30.0)),
                    );
                    ui.label(RichText::new(t.title).size(20.0).family(bold()));
                    ui.add_space(12.0);

                    if ui
                        .button(format!("🗋  {}", t.new))
                        .on_hover_text("Ctrl+N")
                        .clicked()
                    {
                        self.request(Pending::New, &ctx);
                    }
                    if ui
                        .button(format!("🗁  {}", t.open))
                        .on_hover_text("Ctrl+O")
                        .clicked()
                    {
                        self.request(Pending::Open, &ctx);
                    }
                    if ui
                        .button(format!("💾  {}", t.save))
                        .on_hover_text("Ctrl+S")
                        .clicked()
                    {
                        self.save(false);
                    }
                    if ui.button(t.save_as.to_string()).clicked() {
                        self.save(true);
                    }
                    ui.menu_button(format!("📚  {}", t.examples), |ui| {
                        for (i, (name, _)) in EXAMPLES.iter().enumerate() {
                            if ui.button(*name).clicked() {
                                self.request(Pending::Example(i), &ctx);
                            }
                        }
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button(format!("❓  {}", t.help)).clicked() {
                            self.show_help = !self.show_help;
                        }
                        // Setările rămân deschise cât timp se bifează; se închid la click în afară.
                        egui::containers::menu::MenuButton::new(format!("⚙  {}", t.settings))
                            .config(
                                egui::containers::menu::MenuConfig::new()
                                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside),
                            )
                            .ui(ui, |ui| self.settings_menu(ui));
                        ui.menu_button(format!("🌐  {}", self.s.lang.name()), |ui| {
                            for l in Lang::ALL {
                                ui.selectable_value(&mut self.s.lang, l, l.name());
                            }
                        });
                    });
                });
            });
    }

    fn settings_menu(&mut self, ui: &mut Ui) {
        let t = self.t();
        ui.set_min_width(220.0);
        ui.label(RichText::new(t.theme).strong());
        let before = self.s.theme;
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.s.theme, ThemeChoice::System, t.theme_system);
            ui.selectable_value(
                &mut self.s.theme,
                ThemeChoice::Light,
                format!("☀ {}", t.theme_light),
            );
            ui.selectable_value(
                &mut self.s.theme,
                ThemeChoice::Dark,
                format!("🌙 {}", t.theme_dark),
            );
        });
        if before != self.s.theme {
            self.apply_theme(ui.ctx());
        }
        ui.separator();
        ui.label(RichText::new(t.field).strong());
        let (c, r) = (self.s.cols, self.s.rows);
        ui.add(egui::Slider::new(&mut self.s.cols, 5..=40).text(t.columns));
        ui.add(egui::Slider::new(&mut self.s.rows, 5..=40).text(t.rows));
        if (c, r) != (self.s.cols, self.s.rows) {
            self.reset();
        }
        ui.checkbox(&mut self.s.show_grid, t.show_grid);
        ui.checkbox(&mut self.s.show_jumps, t.show_jumps);
        ui.separator();
        ui.checkbox(&mut self.s.live_check, t.live_check);
    }

    fn status_bar(&mut self, ui: &mut Ui) {
        let t = self.t();
        let pal = Pal::new(ui.visuals().dark_mode);
        egui::Panel::bottom("status")
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().panel_fill)
                    .inner_margin(Margin::symmetric(14, 6)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (icon, color, text) = self.status_text(&pal);
                    ui.label(RichText::new(format!("{icon}  {text}")).color(color));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let w = &self.world;
                        ui.label(
                            RichText::new(format!("{}: {}", t.lines_drawn, w.lines.len()))
                                .color(pal.muted),
                        );
                        ui.separator();
                        ui.label(
                            RichText::new(format!("{}: {}", t.actions, w.step_count))
                                .color(pal.muted),
                        );
                        ui.separator();
                        ui.label(
                            RichText::new(format!("{}: {}", t.direction, t.dir(w.dir)))
                                .color(pal.muted),
                        );
                        ui.separator();
                        ui.label(
                            RichText::new(format!("{}: ({}, {})", t.position, w.pos.0, w.pos.1))
                                .color(pal.muted),
                        );
                    });
                });
            });
    }

    fn status_text(&self, pal: &Pal) -> (&'static str, Color32, String) {
        let t = self.t();
        let lang = self.s.lang;
        match &self.status {
            Status::Ready => ("●", pal.muted, t.ready.into()),
            Status::Running => ("▶", pal.accent, t.running.into()),
            Status::Paused => ("⏸", pal.accent, t.paused.into()),
            Status::Finished => ("✔", pal.ok, t.finished.into()),
            Status::Stopped => ("⏹", pal.muted, t.stopped.into()),
            Status::Checked(0, 0) => ("✔", pal.ok, t.no_errors.into()),
            Status::Checked(0, w) => ("✔", pal.ok, (t.summary)(0, *w)),
            Status::Checked(e, w) => ("✖", pal.err, (t.summary)(*e, *w)),
            Status::Compile(e, total) => {
                let more = if *total > 1 {
                    format!("  ({})", (t.summary)(*total, 0))
                } else {
                    String::new()
                };
                (
                    "✖",
                    pal.err,
                    format!(
                        "{} {}: {}{more}",
                        t.error_line,
                        e.line,
                        t.compile_error(lang, &e.kind)
                    ),
                )
            }
            Status::Runtime(e, line) => (
                "✖",
                pal.err,
                format!("{} {line}: {}", t.error_line, t.runtime_error(lang, *e)),
            ),
            Status::File(msg) => ("✖", pal.err, msg.clone()),
        }
    }

    fn editor_panel(&mut self, ui: &mut Ui) {
        let t = self.t();
        let kw = &t.kw;
        let ctx = ui.ctx().clone();
        let pal = Pal::new(ui.visuals().dark_mode);
        let editable = self.mode == Mode::Idle;

        egui::Panel::left("editor_panel")
            .default_size(430.0)
            .size_range(300.0..=900.0)
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().panel_fill)
                    .inner_margin(Margin::same(12)),
            )
            .show(ui, |ui| {
                let (title, format) = ui
                    .horizontal(|ui| {
                        let title = ui.label(RichText::new(t.program).size(16.0).family(bold()));
                        ui.label(
                            RichText::new(format!(
                                "{}{}",
                                self.file_name(),
                                if self.s.dirty { " •" } else { "" }
                            ))
                            .color(pal.muted),
                        );
                        let format = ui
                            .with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.add_enabled(
                                    editable,
                                    egui::Button::new(format!("⇥  {}", t.format)),
                                )
                                .on_hover_text("Ctrl+Shift+F")
                                .clicked()
                            })
                            .inner;
                        (title.id, format)
                    })
                    .inner;
                if format {
                    self.format_code(&ctx);
                }
                ui.add_space(6.0);

                // Butoane pentru inserarea comenzilor.
                let snippets: [(String, Color32, Snippet); 8] = [
                    (kw.step.into(), pal.action, Snippet::word(kw.step)),
                    (kw.jump.into(), pal.action, Snippet::word(kw.jump)),
                    (kw.rotate.into(), pal.action, Snippet::word(kw.rotate)),
                    (
                        format!("{} N", kw.repeat),
                        pal.control,
                        Snippet::block(
                            &format!("{} 4 {}", kw.repeat, kw.times),
                            kw.step,
                            kw.end_repeat,
                        ),
                    ),
                    (
                        kw.while_.into(),
                        pal.control,
                        Snippet::block(
                            &format!("{} {} {}", kw.while_, kw.not, kw.border),
                            kw.step,
                            kw.end_cycle,
                        ),
                    ),
                    (kw.if_.into(), pal.control, Snippet::if_else(kw, kw.border)),
                    (
                        kw.procedure.into(),
                        pal.control,
                        Snippet::block(
                            &format!("{} {}", kw.procedure, kw.name),
                            kw.step,
                            kw.end_procedure,
                        ),
                    ),
                    (
                        kw.execute.into(),
                        pal.control,
                        Snippet {
                            before: format!("{} ", kw.execute),
                            demo: kw.name.into(),
                            after: String::new(),
                        },
                    ),
                ];
                // Primul rînd: acțiunile; al doilea: ciclurile, condiția și procedurile.
                ui.add_enabled_ui(editable, |ui| {
                    ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                    let (actions, control) = snippets.split_at(3);
                    for row in [actions, control] {
                        ui.horizontal_wrapped(|ui| {
                            for (label, color, snippet) in row {
                                let b = egui::Button::new(
                                    RichText::new(label)
                                        .color(*color)
                                        .family(FontFamily::Monospace),
                                );
                                if ui.add(b).on_hover_text(snippet.text()).clicked() {
                                    self.insert_snippet(snippet);
                                }
                            }
                        });
                    }
                });
                ui.add_space(8.0);
                self.problems_panel(ui, &pal);

                let frame_fill = if ui.visuals().dark_mode {
                    Color32::from_rgb(0x17, 0x1A, 0x20)
                } else {
                    Color32::WHITE
                };
                egui::Frame::new()
                    .fill(frame_fill)
                    .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                    .corner_radius(10)
                    .inner_margin(Margin::same(4))
                    .show(ui, |ui| {
                        egui::ScrollArea::both()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                self.code_editor(ui, &pal, editable, title);
                            });
                    });
            });
    }

    fn problems_panel(&mut self, ui: &mut Ui, pal: &Pal) {
        let t = self.t();
        let diags = self.diagnostics();
        let (errors, warnings) = (self.analysis.errors.len(), self.analysis.warnings.len());
        let mut fix = None;
        // Aproximativ o treime din înălțimea disponibilă sub butoane.
        let height = (ui.available_height() * 0.32).clamp(90.0, 280.0);
        egui::Panel::bottom("problems")
            .resizable(false)
            .exact_size(height)
            .frame(egui::Frame::new().inner_margin(Margin {
                left: 0,
                right: 0,
                top: 8,
                bottom: 0,
            }))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(t.problems).size(15.0).family(bold()));
                    if errors > 0 {
                        ui.label(RichText::new(format!("✖ {errors}")).color(pal.err));
                    }
                    if warnings > 0 {
                        ui.label(RichText::new(format!("⚠ {warnings}")).color(pal.warn));
                    }
                });
                ui.add_space(2.0);
                if self.analyzed.is_none() {
                    ui.label(RichText::new(format!("F7 — {}", t.check)).color(pal.muted));
                    return;
                }
                if diags.is_empty() {
                    ui.label(RichText::new(format!("✔  {}", t.no_problems)).color(pal.ok));
                    return;
                }
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for d in &diags {
                            ui.horizontal_top(|ui| {
                                let (icon, color) = if d.error {
                                    ("✖", pal.err)
                                } else {
                                    ("⚠", pal.warn)
                                };
                                ui.label(RichText::new(icon).color(color));
                                ui.vertical(|ui| {
                                    let text = RichText::new(format!(
                                        "{} {}: {}",
                                        t.line_short, d.line, d.message
                                    ))
                                    .color(pal.text);
                                    let r = ui
                                        .add(egui::Label::new(text).wrap().sense(Sense::click()))
                                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                                    if r.clicked() {
                                        self.jump_to = Some(d.span.clone());
                                    }
                                    if let Some(h) = &d.hint {
                                        let b = ui
                                            .small_button(
                                                RichText::new(format!("{} → {h}", t.fix))
                                                    .color(pal.ok),
                                            )
                                            .on_hover_text(format!("{} „{h}”?", t.did_you_mean));
                                        if b.clicked() {
                                            fix = Some((d.span.clone(), h.clone()));
                                        }
                                    }
                                });
                            });
                            ui.add_space(2.0);
                        }
                    });
            });
        if let Some((span, hint)) = fix {
            self.apply_fix(span, &hint);
        }
    }

    /// Pune selecția `self.cursor` în starea editorului.
    fn store_cursor(&self, ctx: &egui::Context) {
        if let Some(sel) = &self.cursor {
            let id = egui::Id::unique(EDITOR_ID);
            let mut state = egui::text_edit::TextEditState::load(ctx, id).unwrap_or_default();
            state.cursor.set_char_range(Some(CCursorRange::two(
                CCursor::new(sel.start),
                CCursor::new(sel.end),
            )));
            state.store(ctx, id);
        }
    }

    /// Lista de variante sub comanda scrisă: se deschide când elevul scrie și
    /// se închide când cursorul pleacă sau editorul pierde focusul.
    fn completion_list(&mut self, ui: &Ui, out: &egui::text_edit::TextEditOutput, edited: bool) {
        if out.response.changed() && !edited {
            self.update_completion();
        }
        let Some(c) = &mut self.completion else {
            return;
        };
        let on_list = c
            .rect
            .zip(ui.input(|i| i.pointer.hover_pos()))
            .is_some_and(|(r, p)| r.contains(p));
        let focused = out.response.response.has_focus() || self.focus_editor;
        if self.cursor != Some(c.range.end..c.range.end) || (!focused && !on_list) {
            self.completion = None;
            return;
        }

        let at = out.galley.pos_from_cursor(CCursor::new(c.range.start));
        let pos = out.galley_pos + at.left_bottom().to_vec2() + vec2(0.0, 2.0);
        let mut clicked = None;
        let area = egui::Area::new(out.response.response.id.with("completion"))
            .order(egui::Order::Foreground)
            .fixed_pos(pos)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    // Fiecare variantă pe un singur rând, oricât de îngust e panoul.
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    for (i, s) in c.items.iter().enumerate() {
                        let text = RichText::new(s.label()).family(FontFamily::Monospace);
                        if ui.selectable_label(i == c.selected, text).clicked() {
                            clicked = Some(i);
                        }
                    }
                });
            });
        c.rect = Some(area.response.rect);
        if let Some(i) = clicked {
            self.accept_completion(i);
        }
    }

    fn code_editor(&mut self, ui: &mut Ui, pal: &Pal, editable: bool, label: egui::Id) {
        let mono = FontId::monospace(15.0);
        let gutter = 44.0;
        let mut layouter = |ui: &Ui, buf: &dyn TextBuffer, _wrap: f32| {
            let text = buf.as_str();
            let mut job = LayoutJob::default();
            for (range, class) in lang::highlight(text) {
                job.append(
                    &text[range],
                    0.0,
                    TextFormat {
                        font_id: mono.clone(),
                        color: pal.token(class),
                        italics: class == TokenClass::Comment,
                        ..Default::default()
                    },
                );
            }
            job.wrap.max_width = f32::INFINITY;
            ui.fonts_mut(|f| f.layout_job(job))
        };

        let id = egui::Id::unique(EDITOR_ID);
        let mut scroll_to = None;
        if let Some(span) = self.jump_to.take() {
            let a = char_index(&self.s.code, span.start);
            let b = char_index(&self.s.code, span.end);
            self.cursor = Some(a..b);
            self.focus_editor = true;
            scroll_to = Some(a);
        }

        let mut edited = false;
        // Tastele listei de variante: ↑/↓ aleg, Enter/Tab inserează, Esc închide.
        let focused = ui.memory(|m| m.has_focus(id));
        if !editable {
            self.completion = None;
        }
        // La Esc egui scoate deja focusul din editor, la începutul cadrului.
        let esc = ui.input(|i| i.key_pressed(Key::Escape));
        if (focused || esc)
            && let Some(c) = &mut self.completion
        {
            let n = c.items.len();
            let (up, down, accept, close) = ui.input_mut(|i| {
                (
                    i.consume_key(Modifiers::NONE, Key::ArrowUp),
                    i.consume_key(Modifiers::NONE, Key::ArrowDown),
                    i.consume_key(Modifiers::NONE, Key::Enter)
                        || i.consume_key(Modifiers::NONE, Key::Tab),
                    i.consume_key(Modifiers::NONE, Key::Escape),
                )
            });
            if up {
                c.selected = (c.selected + n - 1) % n;
            }
            if down {
                c.selected = (c.selected + 1) % n;
            }
            let selected = c.selected;
            if close {
                self.completion = None;
                self.focus_editor = true;
            } else if accept {
                self.accept_completion(selected);
                self.store_cursor(ui.ctx());
                edited = true;
            }
        }

        // Enter e tratat aici, ca rândul nou să primească indentarea blocului.
        if editable
            && ui.memory(|m| m.has_focus(id))
            && let Some(mut state) = egui::text_edit::TextEditState::load(ui.ctx(), id)
            && let Some(range) = state.cursor.char_range()
            && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter))
        {
            let r = range.as_sorted_char_range();
            let at = self.new_line(r.start.into()..r.end.into());
            state
                .cursor
                .set_char_range(Some(CCursorRange::one(CCursor::new(at))));
            state.store(ui.ctx(), id);
            edited = true;
        }

        let bg = ui.painter().add(Shape::Noop);
        let out = egui::TextEdit::multiline(&mut self.s.code)
            .id(id)
            .font(mono.clone())
            .code_editor()
            .frame(egui::Frame::NONE.inner_margin(Margin {
                left: gutter as i8,
                right: 8,
                top: 6,
                bottom: 6,
            }))
            .lock_focus(true)
            .interactive(editable)
            .desired_width(f32::INFINITY)
            .desired_rows(30)
            .layouter(&mut layouter)
            .show(ui);
        let _ = out.response.response.clone().labelled_by(label);

        if out.response.changed() || edited {
            self.s.dirty = true;
            if !matches!(self.status, Status::Ready) {
                self.status = Status::Ready;
                self.error_line = None;
            }
        }
        if let Some(r) = out.cursor_range {
            let r = r.as_sorted_char_range();
            self.cursor = Some(r.start.into()..r.end.into());
        }
        if std::mem::take(&mut self.focus_editor) {
            // Fără focus, egui strânge selecția la cursor: o pune la loc.
            self.store_cursor(ui.ctx());
            out.response.response.request_focus();
        }
        self.completion_list(ui, &out, edited);

        let diags = if self.analyzed.as_ref() == Some(&self.s.code) {
            self.diagnostics()
        } else {
            Vec::new()
        };
        let error_lines: std::collections::HashSet<usize> =
            diags.iter().filter(|d| d.error).map(|d| d.line).collect();
        let warning_lines: std::collections::HashSet<usize> =
            diags.iter().filter(|d| !d.error).map(|d| d.line).collect();

        // Numerele rândurilor și evidențierea rândului curent / cu eroare.
        let painter = ui.painter().clone();
        let full = out.response.rect;
        let mut line = 1;
        let mut first_row = true;
        let mut highlight = Vec::new();
        for row in &out.galley.rows {
            let r = row.rect().translate(out.galley_pos.to_vec2());
            if first_row {
                let marked = error_lines.contains(&line) || Some(line) == self.error_line;
                let color = if Some(line) == self.line || marked {
                    pal.text
                } else {
                    pal.muted
                };
                painter.text(
                    pos2(full.left() + gutter - 12.0, r.center().y),
                    Align2::RIGHT_CENTER,
                    line.to_string(),
                    FontId::monospace(12.5),
                    color,
                );
                if Some(line) == self.line {
                    painter.text(
                        pos2(full.left() + 4.0, r.center().y),
                        Align2::LEFT_CENTER,
                        "▶",
                        FontId::proportional(11.0),
                        pal.accent,
                    );
                } else if marked {
                    painter.circle_filled(pos2(full.left() + 9.0, r.center().y), 4.0, pal.err);
                } else if warning_lines.contains(&line) {
                    let c = pos2(full.left() + 9.0, r.center().y);
                    let tri = vec![c + vec2(0.0, -4.5), c + vec2(4.5, 3.5), c + vec2(-4.5, 3.5)];
                    painter.add(Shape::convex_polygon(tri, pal.warn, Stroke::NONE));
                }
            }
            let fill = if Some(line) == self.error_line || error_lines.contains(&line) {
                Some(pal.error_line)
            } else if Some(line) == self.line {
                Some(pal.current_line)
            } else {
                None
            };
            if let Some(fill) = fill {
                let rect = Rect::from_x_y_ranges(
                    full.left()..=full.right().max(ui.clip_rect().right()),
                    r.y_range(),
                );
                highlight.push(Shape::rect_filled(rect, 3.0, fill));
            }
            first_row = row.ends_with_newline;
            if row.ends_with_newline {
                line += 1;
            }
        }
        ui.painter().set(bg, Shape::Vec(highlight));

        // Sublinierea ondulată a cuvintelor greșite, cu explicație la hover.
        let gp = out.galley_pos.to_vec2();
        let hover = out.response.response.hover_pos();
        let mut tooltip = None;
        for d in &diags {
            let a = char_index(&self.s.code, d.span.start);
            let b = char_index(&self.s.code, d.span.end).max(a);
            let ra = out.galley.pos_from_cursor(CCursor::new(a)).translate(gp);
            let rb = out.galley.pos_from_cursor(CCursor::new(b)).translate(gp);
            let same_row = (ra.center().y - rb.center().y).abs() < 1.0;
            let x1 = if same_row && b > a {
                rb.left()
            } else {
                ra.left() + 10.0
            };
            let color = if d.error { pal.err } else { pal.warn };
            wavy_underline(&painter, ra.left(), x1, ra.bottom() - 1.0, color);
            let hit =
                Rect::from_x_y_ranges(ra.left() - 2.0..=x1 + 2.0, ra.top()..=ra.bottom() + 2.0);
            if hover.is_some_and(|p| hit.contains(p)) {
                tooltip = Some(d);
            }
        }
        if let Some(d) = tooltip {
            let t = self.t();
            let _ = out.response.response.clone().on_hover_ui_at_pointer(|ui| {
                ui.label(RichText::new(&d.message).color(if d.error { pal.err } else { pal.warn }));
                if let Some(h) = &d.hint {
                    ui.label(format!("{} „{h}”?", t.did_you_mean));
                }
            });
        }
        if let Some(a) = scroll_to {
            let r = out.galley.pos_from_cursor(CCursor::new(a)).translate(gp);
            ui.scroll_to_rect(r.expand(40.0), None);
        }

        // Derulează la rândul care se execută.
        if let (Some(cur), Mode::Running | Mode::Paused) = (self.line, self.mode) {
            let mut l = 1;
            for row in &out.galley.rows {
                if l == cur {
                    let r = row.rect().translate(out.galley_pos.to_vec2());
                    ui.scroll_to_rect(r.expand(30.0), None);
                    break;
                }
                if row.ends_with_newline {
                    l += 1;
                }
            }
        }
    }

    fn controls(&mut self, ui: &mut Ui) {
        let t = self.t();
        let pal = Pal::new(ui.visuals().dark_mode);
        let now = ui.input(|i| i.time);
        // Se rupe pe două rânduri când fereastra e îngustă.
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().button_padding = vec2(12.0, 7.0);
            if self.mode == Mode::Running {
                let b = egui::Button::new(
                    RichText::new(format!("⏸  {}", t.pause))
                        .color(Color32::WHITE)
                        .strong(),
                )
                .fill(pal.accent);
                if ui.add(b).on_hover_text("F5").clicked() {
                    self.pause();
                }
            } else {
                let b = egui::Button::new(
                    RichText::new(format!("▶  {}", t.run))
                        .color(Color32::WHITE)
                        .strong(),
                )
                .fill(pal.accent);
                if ui.add(b).on_hover_text("F5").clicked() {
                    self.run();
                }
            }
            if ui
                .button(format!("⏭  {}", t.step_once))
                .on_hover_text("F6")
                .clicked()
            {
                self.step_once(now);
            }
            if ui
                .add_enabled(
                    self.mode != Mode::Idle,
                    egui::Button::new(format!("⏹  {}", t.stop)),
                )
                .on_hover_text("Esc")
                .clicked()
            {
                self.stop();
            }
            if ui.button(format!("↺  {}", t.reset)).clicked() {
                self.reset();
            }
            if ui
                .button(format!("✔  {}", t.check))
                .on_hover_text("F7")
                .clicked()
            {
                self.check();
            }

            ui.add_space(10.0);
            let label = ui.label(RichText::new(t.speed).color(pal.muted));
            ui.spacing_mut().slider_width = 90.0;
            let slider = ui.add_enabled(
                !self.s.instant,
                egui::Slider::new(&mut self.s.speed, 1.0..=10.0)
                    .step_by(1.0)
                    .show_value(false),
            );
            let _ = slider.labelled_by(label.id);
            ui.toggle_value(&mut self.s.instant, format!("⚡ {}", t.instant));
        });
    }

    fn field(&self, ui: &mut Ui) {
        let pal = Pal::new(ui.visuals().dark_mode);
        let (resp, painter) = ui.allocate_painter(ui.available_size(), Sense::hover());
        let area = resp.rect;
        let w = &self.world;
        let (cols, rows) = (w.width as f32, w.height as f32);
        let (origin, cell) = field_layout(area, cols, rows);
        let size = vec2(cell * cols, cell * rows);
        let node = |(x, y): (i32, i32)| origin + vec2(x as f32 * cell, y as f32 * cell);
        let field_rect = Rect::from_min_size(origin, size);

        painter.rect_filled(field_rect.expand(18.0), 16.0, pal.field_bg);

        if self.s.show_grid {
            let stroke = Stroke::new(1.0, pal.grid);
            for i in 1..w.width {
                let x = origin.x + i as f32 * cell;
                painter.line_segment(
                    [pos2(x, field_rect.top()), pos2(x, field_rect.bottom())],
                    stroke,
                );
            }
            for j in 1..w.height {
                let y = origin.y + j as f32 * cell;
                painter.line_segment(
                    [pos2(field_rect.left(), y), pos2(field_rect.right(), y)],
                    stroke,
                );
            }
        }
        painter.rect_stroke(
            field_rect,
            2.0,
            Stroke::new(2.0, pal.border),
            egui::StrokeKind::Middle,
        );

        // Coordonatele pe margini, din 5 în 5.
        let label_font = FontId::proportional(10.5);
        let step = if cell < 14.0 { 10 } else { 5 };
        for i in (0..=w.width).step_by(step) {
            painter.text(
                node((i, 0)) - vec2(0.0, 22.0),
                Align2::CENTER_CENTER,
                i.to_string(),
                label_font.clone(),
                pal.muted,
            );
        }
        for j in (0..=w.height).step_by(step) {
            painter.text(
                node((0, j)) - vec2(22.0, 0.0),
                Align2::CENTER_CENTER,
                j.to_string(),
                label_font.clone(),
                pal.muted,
            );
        }

        let progress = self.anim.as_ref().map(|a| {
            let now = ui.input(|i| i.time);
            ((now - a.start) / a.dur).clamp(0.0, 1.0) as f32
        });

        if self.s.show_jumps {
            let stroke = Stroke::new(1.5, pal.muted.gamma_multiply(0.7));
            for &(a, b) in &w.jumps {
                painter.extend(Shape::dashed_line(&[node(a), node(b)], stroke, 4.0, 4.0));
            }
        }

        let thick = (cell * 0.14).clamp(2.0, 6.0);
        let ink = Stroke::new(thick, pal.ink);
        let partial = match (&self.anim, progress) {
            (Some(a), Some(p)) if a.mv.action == Action::Step && a.new_line => Some((a.mv, p)),
            _ => None,
        };
        let n = w.lines.len() - usize::from(partial.is_some());
        for &(a, b) in &w.lines[..n] {
            painter.line_segment([node(a), node(b)], ink);
            painter.circle_filled(node(a), thick / 2.0, pal.ink);
            painter.circle_filled(node(b), thick / 2.0, pal.ink);
        }
        if let Some((mv, p)) = partial {
            let a = node(mv.from);
            let b = a + (node(mv.to) - a) * ease(p);
            painter.line_segment([a, b], ink);
            painter.circle_filled(a, thick / 2.0, pal.ink);
            painter.circle_filled(b, thick / 2.0, pal.ink);
        }

        // Cangurul.
        let (pos, dir) = match (&self.anim, progress) {
            (Some(a), Some(p)) => {
                let from = node(a.mv.from);
                let to = node(a.mv.to);
                let mut pos = from + (to - from) * ease(p);
                if a.mv.action == Action::Jump {
                    pos.y -= (p * std::f32::consts::PI).sin() * cell * 0.5;
                }
                let dir = if a.mv.action == Action::Rotate && p < 0.5 {
                    a.mv.dir_before
                } else {
                    a.mv.dir_after
                };
                (pos, dir)
            }
            _ => (node(w.pos), w.dir),
        };
        let here = node(w.pos);
        let sprite = sprite_size(cell);

        painter.circle_filled(here, (thick * 1.6).max(4.0), pal.accent);
        let (dx, dy) = dir.delta();
        let d = vec2(dx as f32, dy as f32);
        let tip = here + d * (cell * 0.45).max(10.0);
        let side = vec2(-d.y, d.x) * 5.0;
        let base = tip - d * 8.0;
        painter.add(Shape::convex_polygon(
            vec![tip, base + side, base - side],
            pal.accent,
            Stroke::NONE,
        ));

        let tex = if self.crashed {
            &self.sprites.tired
        } else {
            self.sprites.for_dir(dir)
        };
        let rect = Rect::from_center_size(pos - vec2(0.0, sprite * 0.38), Vec2::splat(sprite));
        painter.add(Shape::ellipse_filled(
            pos + vec2(0.0, 1.0),
            vec2(sprite * 0.25, sprite * 0.07),
            Color32::from_black_alpha(50),
        ));
        painter.image(
            tex.at(sprite * painter.pixels_per_point()).id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }

    fn help_window(&mut self, ctx: &egui::Context) {
        let t = self.t();
        let pal = Pal::new(ctx.global_style().visuals.dark_mode);
        let mut open = self.show_help;
        egui::Window::new(format!("❓ {}", t.help))
            .open(&mut open)
            .default_size([560.0, 600.0])
            .default_pos(ctx.content_rect().right_top() + vec2(-600.0, 70.0))
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (syntax, desc) in t.help_text {
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(*syntax)
                                .family(FontFamily::Monospace)
                                .color(pal.control)
                                .size(14.5),
                        );
                        ui.label(*desc);
                    }
                    ui.add_space(10.0);
                    ui.separator();
                    egui::Grid::new("kw_table")
                        .striped(true)
                        .spacing([18.0, 6.0])
                        .show(ui, |ui| {
                            for l in Lang::ALL {
                                ui.label(RichText::new(l.name()).strong());
                            }
                            ui.end_row();
                            let rows: [fn(&crate::i18n::Keywords) -> &'static str; 15] = [
                                |k| k.step,
                                |k| k.jump,
                                |k| k.rotate,
                                |k| k.repeat,
                                |k| k.times,
                                |k| k.end_repeat,
                                |k| k.while_,
                                |k| k.end_cycle,
                                |k| k.if_,
                                |k| k.else_,
                                |k| k.not,
                                |k| k.border,
                                |k| k.line,
                                |k| k.procedure,
                                |k| k.execute,
                            ];
                            for f in rows {
                                for l in Lang::ALL {
                                    ui.label(
                                        RichText::new(f(&l.t().kw)).family(FontFamily::Monospace),
                                    );
                                }
                                ui.end_row();
                            }
                        });
                    ui.add_space(10.0);
                    ui.separator();
                    ui.label(RichText::new(t.shortcuts).color(pal.muted));
                    ui.add_space(6.0);
                    ui.label(RichText::new(t.about).color(pal.muted));
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(format!("{}:", t.author)).color(pal.muted));
                        ui.label(RichText::new("Nicolae Postica").strong());
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.hyperlink_to(
                            RichText::new("nicolae.postica2@gmail.com").underline(),
                            "mailto:nicolae.postica2@gmail.com",
                        );
                        ui.label(RichText::new("·").color(pal.muted));
                        ui.hyperlink_to(
                            RichText::new("github.com/nazdravan/cangurul").underline(),
                            "https://github.com/nazdravan/cangurul",
                        );
                    });
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(concat!("v", env!("CARGO_PKG_VERSION")))
                            .color(pal.muted)
                            .small(),
                    );
                });
            });
        self.show_help = open;
    }

    fn unsaved_dialog(&mut self, ctx: &egui::Context) {
        let Some(p) = self.pending else { return };
        let t = self.t();
        let mut choice = None;
        egui::Modal::new(egui::Id::unique("unsaved")).show(ctx, |ui| {
            ui.set_width(360.0);
            ui.heading(t.unsaved_title);
            ui.add_space(6.0);
            ui.label(t.unsaved_text);
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button(format!("💾  {}", t.save)).clicked() {
                    choice = Some(0);
                }
                if ui.button(t.discard).clicked() {
                    choice = Some(1);
                }
                if ui.button(t.cancel).clicked() {
                    choice = Some(2);
                }
            });
        });
        match choice {
            Some(0) => {
                self.pending = None;
                if self.save(false) {
                    self.perform(p, ctx);
                }
            }
            Some(1) => {
                self.pending = None;
                self.s.dirty = false;
                self.perform(p, ctx);
            }
            Some(_) => self.pending = None,
            None => {}
        }
    }
}

impl eframe::App for CangurApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.shortcuts(ctx);
        self.refresh_analysis();
        self.drive(ctx);
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.top_bar(ui);
        self.status_bar(ui);
        self.editor_panel(ui);
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().extreme_bg_color)
                    .inner_margin(Margin::same(14)),
            )
            .show(ui, |ui| {
                self.controls(ui);
                ui.add_space(8.0);
                self.field(ui);
            });

        self.help_window(&ctx);
        self.unsaved_dialog(&ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, STORAGE_KEY, &self.s);
    }
}

/// Indexul caracterului (pentru cursorul editorului) la o poziție în octeți.
fn char_index(s: &str, byte: usize) -> usize {
    let mut b = byte.min(s.len());
    while !s.is_char_boundary(b) {
        b -= 1;
    }
    s[..b].chars().count()
}

/// Poziția cursorului (în caractere) după o formatare care schimbă doar
/// spațiile și rândurile: rămâne lângă același caracter vizibil.
fn respaced_cursor(old: &str, new: &str, at: usize) -> usize {
    let visible = old.chars().take(at).filter(|c| !c.is_whitespace()).count();
    // Lipit de textul din stânga rămâne după el; altfel trece la textul următor.
    let after_text = at > 0 && old.chars().nth(at - 1).is_some_and(|c| !c.is_whitespace());
    let mut seen = 0;
    for (i, c) in new.chars().enumerate() {
        if seen == visible && (after_text || !c.is_whitespace()) {
            return i;
        }
        if !c.is_whitespace() {
            seen += 1;
        }
    }
    new.chars().count()
}

/// Pune pe rândurile `lines` indentarea dată de formatare; restul rămâne.
fn reindent_lines(code: &str, lines: std::ops::RangeInclusive<usize>) -> String {
    let formatted = lang::format(code);
    let out: Vec<&str> = code
        .split('\n')
        .zip(formatted.split('\n'))
        .enumerate()
        .map(|(i, (old, f))| if lines.contains(&i) { f } else { old })
        .collect();
    out.join("\n")
}

/// Poziția cursorului (în caractere) după reindentare: rămâne pe același
/// caracter din rând; din spațiile de la început trece la primul cuvânt.
fn reindented_cursor(old: &str, new: &str, at: usize) -> usize {
    let indent = |l: &str| l.chars().take_while(|c| c.is_whitespace()).count();
    let (mut old_start, mut new_start) = (0, 0);
    for (o, n) in old.split('\n').zip(new.split('\n')) {
        let (old_len, new_len) = (o.chars().count(), n.chars().count());
        if at <= old_start + old_len {
            let col = (at - old_start).saturating_sub(indent(o)) + indent(n);
            return new_start + col.min(new_len);
        }
        old_start += old_len + 1;
        new_start += new_len + 1;
    }
    new.chars().count()
}

fn wavy_underline(painter: &egui::Painter, x0: f32, x1: f32, y: f32, color: Color32) {
    let mut points = Vec::new();
    let mut x = x0;
    while x <= x1 {
        let phase = ((x - x0) / 2.0) as i32 % 2;
        points.push(pos2(x, y + if phase == 0 { -1.2 } else { 1.2 }));
        x += 2.0;
    }
    if points.len() >= 2 {
        painter.add(Shape::line(points, Stroke::new(1.3, color)));
    }
}

fn ease(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn bold() -> FontFamily {
    FontFamily::Name("bold".into())
}

/// Fișierele vechi sunt în Windows-1250 (română) sau Windows-1251 (rusă).
pub fn decode_cng(bytes: &[u8]) -> String {
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    let text = match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            let (cyr, _, _) = encoding_rs::WINDOWS_1251.decode(bytes);
            // În rusă cuvintele sunt scrise numai cu chirilice; un text românesc
            // decodat greșit ar amesteca litere latine cu chirilice în același cuvânt.
            let mixed = cyr.split(|c: char| !c.is_alphanumeric()).any(|w| {
                w.chars().any(|c| c.is_ascii_alphabetic())
                    && w.chars().any(|c| ('\u{400}'..='\u{4FF}').contains(&c))
            });
            if mixed {
                encoding_rs::WINDOWS_1250.decode(bytes).0.into_owned()
            } else {
                cyr.into_owned()
            }
        }
    };
    text.replace("\r\n", "\n")
}

fn setup_style(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let data = |b: &'static [u8]| Arc::new(egui::FontData::from_static(b));
    fonts.font_data.insert(
        "noto".into(),
        data(include_bytes!("../assets/fonts/NotoSans-Regular.ttf")),
    );
    fonts.font_data.insert(
        "noto-bold".into(),
        data(include_bytes!("../assets/fonts/NotoSans-Bold.ttf")),
    );
    fonts.font_data.insert(
        "fira".into(),
        data(include_bytes!("../assets/fonts/FiraCode-Regular.ttf")),
    );
    let prop = fonts.families.entry(FontFamily::Proportional).or_default();
    prop.insert(0, "noto".into());
    // Fira Code are săgețile și simbolurile care lipsesc din Noto Sans.
    prop.push("fira".into());
    let mut bold_family = vec!["noto-bold".to_string()];
    bold_family.extend(prop.iter().cloned());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "fira".into());
    fonts.families.insert(bold(), bold_family);
    ctx.set_fonts(fonts);

    ctx.all_styles_mut(|style| {
        use egui::TextStyle::*;
        style.text_styles.insert(Body, FontId::proportional(14.5));
        style.text_styles.insert(Button, FontId::proportional(14.5));
        style.text_styles.insert(Heading, FontId::new(19.0, bold()));
        style.text_styles.insert(Small, FontId::proportional(12.0));
        style.text_styles.insert(Monospace, FontId::monospace(14.5));
        style.spacing.item_spacing = vec2(8.0, 6.0);
        style.spacing.button_padding = vec2(10.0, 5.0);
        style.spacing.interact_size.y = 28.0;
        style.spacing.slider_width = 140.0;
        let r = CornerRadius::same(8);
        let v = &mut style.visuals;
        v.widgets.noninteractive.corner_radius = r;
        v.widgets.inactive.corner_radius = r;
        v.widgets.hovered.corner_radius = r;
        v.widgets.active.corner_radius = r;
        v.widgets.open.corner_radius = r;
        v.window_corner_radius = CornerRadius::same(12);
        v.menu_corner_radius = CornerRadius::same(10);
    });

    ctx.style_mut_of(Theme::Light, |s| {
        let v = &mut s.visuals;
        v.panel_fill = Color32::from_rgb(0xF7, 0xF5, 0xF1);
        v.extreme_bg_color = Color32::from_rgb(0xEE, 0xEB, 0xE4);
        v.window_fill = Color32::from_rgb(0xFF, 0xFE, 0xFB);
        v.widgets.inactive.weak_bg_fill = Color32::from_rgb(0xEA, 0xE6, 0xDE);
        v.widgets.inactive.bg_fill = Color32::from_rgb(0xEA, 0xE6, 0xDE);
        v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0xE0, 0xDB, 0xD0);
        v.selection.bg_fill = Color32::from_rgb(0xF9, 0xC8, 0x9B);
        v.selection.stroke = Stroke::new(1.0, Color32::from_rgb(0xC2, 0x55, 0x0C));
        v.hyperlink_color = Color32::from_rgb(0xC2, 0x55, 0x0C);
    });
    ctx.style_mut_of(Theme::Dark, |s| {
        let v = &mut s.visuals;
        v.panel_fill = Color32::from_rgb(0x1C, 0x1F, 0x26);
        v.extreme_bg_color = Color32::from_rgb(0x14, 0x16, 0x1B);
        v.window_fill = Color32::from_rgb(0x22, 0x26, 0x2E);
        v.widgets.inactive.weak_bg_fill = Color32::from_rgb(0x2C, 0x31, 0x3B);
        v.widgets.inactive.bg_fill = Color32::from_rgb(0x2C, 0x31, 0x3B);
        v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x38, 0x3E, 0x4A);
        v.selection.bg_fill = Color32::from_rgb(0x8A, 0x4A, 0x1A);
        v.selection.stroke = Stroke::new(1.0, Color32::from_rgb(0xF0, 0x8A, 0x3C));
        v.hyperlink_color = Color32::from_rgb(0xF0, 0x8A, 0x3C);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cangurul din colțul de sus (0, 0), la mărimea maximă, încape în zona
    /// câmpului (altfel urechile sunt tăiate pe un ecran mare).
    #[test]
    fn kangaroo_fits_above_top_row() {
        for area in [
            Rect::from_min_size(pos2(500.0, 140.0), vec2(1500.0, 1050.0)),
            Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0)),
        ] {
            let (origin, cell) = field_layout(area, 20.0, 20.0);
            let sprite = sprite_size(cell);
            let top = origin.y - sprite * 0.38 - sprite / 2.0;
            assert!(
                top >= area.top(),
                "cangurul iese cu {} px",
                area.top() - top
            );
            assert!(origin.y + cell * 20.0 <= area.bottom());
        }
    }

    #[test]
    fn decodes_legacy_encodings() {
        let rus = encoding_rs::WINDOWS_1251.encode("[\r\nШАГ\r\n]").0;
        assert_eq!(decode_cng(&rus), "[\nШАГ\n]");
        let rom = encoding_rs::WINDOWS_1250
            .encode("[\nREPETĂ 2 ORI PAS SFÎRŞITUL REPETĂRII\n]")
            .0;
        assert_eq!(
            decode_cng(&rom),
            "[\nREPETĂ 2 ORI PAS SFÎRŞITUL REPETĂRII\n]"
        );
        assert!(lang::compile(&decode_cng(&rom)).is_ok());
    }

    #[test]
    fn examples_compile() {
        for (name, src) in EXAMPLES {
            assert!(lang::compile(src).is_ok(), "{name}");
        }
        assert!(lang::compile(DEFAULT_PROGRAM).is_ok());
    }

    /// Teste de interfață fără fereastră: apasă butoanele reale și,
    /// dacă `CANGURUL_SHOTS` e setat, salvează capturi PNG în acel director.
    mod ui {
        use super::*;
        use egui_kittest::Harness;
        use egui_kittest::kittest::Queryable;

        fn harness<'a>() -> Harness<'a, CangurApp> {
            Harness::builder()
                .with_size(vec2(1280.0, 800.0))
                .wgpu()
                .build_eframe(|cc| CangurApp::new(cc, None))
        }

        fn shot(h: &mut Harness<'_, CangurApp>, name: &str) {
            if let Ok(dir) = std::env::var("CANGURUL_SHOTS") {
                h.render()
                    .unwrap()
                    .save(format!("{dir}/{name}.png"))
                    .unwrap();
            }
        }

        fn run_until_idle(h: &mut Harness<'_, CangurApp>) {
            for _ in 0..20_000 {
                h.step();
                if h.state().mode == Mode::Idle && h.state().anim.is_none() {
                    break;
                }
            }
            h.step();
        }

        #[test]
        fn settings_stay_open_after_checkbox() {
            let mut h = harness();
            h.run_steps(3);
            h.get_by_label("⚙  Setări").click();
            h.run_steps(3);
            let grid = h.state().s.show_grid;
            h.get_by_label("Arată grila").click();
            h.run_steps(3);
            assert_eq!(h.state().s.show_grid, !grid);
            h.get_by_label("Arată salturile").click();
            h.run_steps(3);
            assert!(
                h.query_by_label("Arată grila").is_some(),
                "setările s-au închis"
            );
        }

        #[test]
        fn help_shows_author_links_apart() {
            let mut h = harness();
            h.ctx.set_theme(ThemePreference::Light);
            h.state_mut().show_help = true;
            h.run_steps(3);
            h.get_by_role_and_label(egui::accesskit::Role::Link, "nicolae.postica2@gmail.com");
            h.get_by_role_and_label(egui::accesskit::Role::Link, "github.com/nazdravan/cangurul")
                .scroll_to_me();
            h.run_steps(3);
            shot(&mut h, "help-author");
        }

        #[test]
        fn run_example_from_ui() {
            let mut h = harness();
            h.ctx.set_theme(ThemePreference::Light);
            h.run_steps(3);
            shot(&mut h, "1-start");

            h.state_mut().s.code = EXAMPLES[0].1.into();
            h.get_by_label("▶  Execută").click();
            h.run_steps(40);
            assert!(h.state().mode == Mode::Running);
            assert!(h.state().line.is_some());
            shot(&mut h, "2-running");

            h.state_mut().s.instant = true;
            run_until_idle(&mut h);
            assert!(matches!(h.state().status, Status::Finished));
            assert!(h.state().world.lines.len() > 50);
            shot(&mut h, "3-finished");
        }

        #[test]
        fn snippet_buttons_insert_before_closing_bracket() {
            let mut h = harness();
            h.state_mut().s.code = "[\nPAS\n]\n".into();
            h.run_steps(2);
            h.get_by_label("ROTIRE").click();
            h.run_steps(2);
            h.get_by_label("REPETĂ N").click();
            h.run_steps(2);
            // Cursorul rămâne în editor după click pe buton.
            let editor = egui::Id::unique(EDITOR_ID);
            assert!(h.ctx.memory(|m| m.has_focus(editor)));
            assert_eq!(
                h.state().s.code,
                "[\nPAS\nROTIRE\nREPETĂ 4 ORI\n    PAS\nSFÎRȘITUL REPETĂRII\n]\n"
            );
            assert!(h.state().s.dirty);
            assert!(lang::compile(&h.state().s.code).is_ok());
            // Exemplul din interior e selectat și e înlocuit de următoarea comandă;
            // comenzile de după păstrează indentarea blocului.
            h.get_by_label("SALT").click();
            h.run_steps(2);
            // După comandă, cursorul e pe un rând nou, cu indentarea blocului.
            assert!(h.state().s.code.contains("    SALT\n    \nSFÎRȘITUL"));
            let at = h.state().s.code.find("    \nSFÎRȘITUL").unwrap() + 4;
            let at = h.state().s.code[..at].chars().count();
            assert_eq!(h.state().cursor, Some(at..at));
            for b in ["ROTIRE", "DACĂ"] {
                h.get_by_label(b).click();
                h.run_steps(2);
            }
            assert_eq!(
                h.state().s.code,
                "[\nPAS\nROTIRE\nREPETĂ 4 ORI\n    SALT\n    ROTIRE\n    DACĂ E_MARGINE\n        ROTIRE\n    ALTFEL\n        PAS\nSFÎRȘITUL REPETĂRII\n]\n"
            );
            // Comanda de după DACĂ e înlocuită, apoi cea de după ALTFEL.
            for b in ["PAS", "EXECUTĂ"] {
                h.get_by_label(b).click();
                h.run_steps(2);
            }
            assert_eq!(
                h.state().s.code,
                "[\nPAS\nROTIRE\nREPETĂ 4 ORI\n    SALT\n    ROTIRE\n    DACĂ E_MARGINE\n        PAS\n    ALTFEL\n        EXECUTĂ Nume\nSFÎRȘITUL REPETĂRII\n]\n"
            );
            let sel = h.state().cursor.clone().unwrap();
            assert_eq!(
                h.state()
                    .s
                    .code
                    .chars()
                    .skip(sel.start)
                    .take(sel.len())
                    .collect::<String>(),
                "Nume"
            );
        }

        #[test]
        fn enter_keeps_block_indentation() {
            let mut h = harness();
            let src = "[\nREPETĂ 2 ORI\n]\n";
            h.state_mut().s.code = src.into();
            let at = src.find('\n').unwrap() + "\nREPETĂ 2 ORI".chars().count();
            h.state_mut().cursor = Some(at..at);
            h.state_mut().focus_editor = true;
            h.run_steps(2);
            let typed = |h: &mut Harness<'_, CangurApp>, text: &str| {
                h.event(egui::Event::Text(text.into()));
                h.run_steps(1);
                h.key_press(Key::Enter);
                h.run_steps(1);
            };
            typed(&mut h, "");
            typed(&mut h, "PAS");
            assert_eq!(h.state().s.code, "[\nREPETĂ 2 ORI\n    PAS\n    \n]\n");
            // SFÎRȘITUL se aliniază cu REPETĂ după Enter.
            typed(&mut h, "SFÎRȘITUL REPETĂRII");
            assert_eq!(
                h.state().s.code,
                "[\nREPETĂ 2 ORI\n    PAS\nSFÎRȘITUL REPETĂRII\n\n]\n"
            );
            let at = h.state().s.code.chars().count() - 3;
            assert_eq!(h.state().cursor, Some(at..at));
        }

        #[test]
        fn completion_while_typing() {
            let mut h = harness();
            h.ctx.set_theme(ThemePreference::Light);
            h.state_mut().s.code = "[\n\n]\n".into();
            h.state_mut().cursor = Some(2..2);
            h.state_mut().focus_editor = true;
            h.run_steps(2);
            let labels = |h: &Harness<'_, CangurApp>| -> Vec<String> {
                h.state()
                    .completion
                    .as_ref()
                    .map(|c| c.items.iter().map(Snippet::label).collect())
                    .unwrap_or_default()
            };

            h.event(egui::Event::Text("cit".into()));
            h.run_steps(2);
            assert_eq!(
                labels(&h),
                [
                    "CÎT NU E_MARGINE …",
                    "CÎT E_MARGINE …",
                    "CÎT NU E_LINIE …",
                    "CÎT E_LINIE …"
                ]
            );
            shot(&mut h, "10-completion");
            h.key_press(Key::ArrowDown);
            h.run_steps(1);
            h.key_press(Key::Enter);
            h.run_steps(2);
            assert_eq!(
                h.state().s.code,
                "[\nCÎT E_MARGINE\n    PAS\nSFÎRȘITUL CICLULUI\n]\n"
            );
            assert!(h.state().completion.is_none());

            // În interiorul ciclului: „s” înlocuiește PAS selectat; sfârșitul
            // ciclului e propus înaintea celorlalte.
            h.event(egui::Event::Text("s".into()));
            h.run_steps(2);
            assert_eq!(
                labels(&h),
                [
                    "SALT",
                    "SFÎRȘITUL CICLULUI",
                    "SFÎRȘITUL REPETĂRII",
                    "SFÎRȘITUL PROCEDURII"
                ]
            );
            h.key_press(Key::Tab);
            h.run_steps(2);
            assert_eq!(
                h.state().s.code,
                "[\nCÎT E_MARGINE\n    SALT\n    \nSFÎRȘITUL CICLULUI\n]\n"
            );

            // Esc închide lista; Enter face atunci rând nou.
            h.event(egui::Event::Text("PA".into()));
            h.run_steps(2);
            assert_eq!(labels(&h), ["PAS"]);
            h.key_press(Key::Escape);
            h.run_steps(1);
            assert!(h.state().completion.is_none());
            h.key_press(Key::Enter);
            h.run_steps(1);

            // După o condiție scrisă întreagă, Enter face rând nou.
            h.event(egui::Event::Text("DACĂ E_MARGINE".into()));
            h.run_steps(2);
            assert!(h.state().completion.is_none());
            h.key_press(Key::Enter);
            h.run_steps(1);
            assert_eq!(
                h.state().s.code,
                "[\nCÎT E_MARGINE\n    SALT\n    PA\n    DACĂ E_MARGINE\n        \nSFÎRȘITUL CICLULUI\n]\n"
            );
        }

        #[test]
        fn completion_click() {
            let mut h = harness();
            h.state_mut().s.code = "PROCEDURA Patrat\nPAS\nSFÎRȘITUL PROCEDURII\n[\n\n]\n".into();
            let at = h.state().s.code.chars().count() - 3;
            h.state_mut().cursor = Some(at..at);
            h.state_mut().focus_editor = true;
            h.run_steps(2);
            h.event(egui::Event::Text("exe".into()));
            h.run_steps(2);
            h.get_by_label("EXECUTĂ Patrat").click();
            h.run_steps(3);
            assert!(h.state().s.code.ends_with("[\nEXECUTĂ Patrat\n\n]\n"));
            let editor = egui::Id::unique(EDITOR_ID);
            assert!(h.ctx.memory(|m| m.has_focus(editor)));
        }

        #[test]
        fn format_button_and_undo() {
            let mut h = harness();
            h.ctx.set_theme(ThemePreference::Light);
            let src =
                "[\nREPETĂ 4 ORI\nPAS\nDACĂ E_MARGINE ROTIRE ALTFEL PAS\nSFÎRȘITUL REPETĂRII\n]\n";
            let formatted = "[\nREPETĂ 4 ORI\n    PAS\n    DACĂ E_MARGINE\n        ROTIRE\n    ALTFEL\n        PAS\nSFÎRȘITUL REPETĂRII\n]\n";
            h.state_mut().s.code = src.into();
            // Cursorul după ROTIRE rămâne după ROTIRE, acum pe rândul lui.
            let at = src[..src.find(" ALTFEL").unwrap()].chars().count();
            h.state_mut().cursor = Some(at..at);
            h.run_steps(2);
            h.get_by_label("⇥  Formatează").click();
            h.run_steps(2);
            assert_eq!(h.state().s.code, formatted);
            let at = formatted[..formatted.find("\n    ALTFEL").unwrap()]
                .chars()
                .count();
            assert_eq!(h.state().cursor, Some(at..at));
            let editor = egui::Id::unique(EDITOR_ID);
            assert!(h.ctx.memory(|m| m.has_focus(editor)));
            shot(&mut h, "9-formatted");

            // Formatarea se anulează cu un singur Ctrl+Z.
            h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
            h.run_steps(2);
            assert_eq!(h.state().s.code, src);

            h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::F);
            h.run_steps(2);
            assert_eq!(h.state().s.code, formatted);
        }

        #[test]
        fn live_analysis_and_quick_fix() {
            let mut h = harness();
            h.ctx.set_theme(ThemePreference::Light);
            h.state_mut().s.code = "PROCEDURA Patrat\nREPETĂ 4 ORI\n    PAS\n    ROTRE\nSFÎRȘITUL REPETĂRII\nSFÎRȘITUL PROCEDURII\n\n[\nSARI\nEXECUTĂ Patat\nCÎT NU E_MARGINE\n    ROTIRE\nSFÎRȘITUL CICLULUI\n]\n".into();
            h.run_steps(3);
            // Analiza rulează singură, fără „Verifică”.
            let errs: Vec<usize> = h.state().analysis.errors.iter().map(|e| e.line).collect();
            assert_eq!(errs, vec![4, 9, 10]);
            // Ciclul fără mișcare și „Patrat” nefolosită (apelul e scris greșit).
            assert_eq!(h.state().analysis.warnings.len(), 2);
            shot(&mut h, "7-problems");

            // Rularea e blocată și arată prima eroare.
            h.get_by_label("▶  Execută").click();
            h.run_steps(2);
            assert!(matches!(h.state().status, Status::Compile(_, 3)));
            assert_eq!(h.state().mode, Mode::Idle);

            for fix in [
                "Corectează → ROTIRE",
                "Corectează → SALT",
                "Corectează → Patrat",
            ] {
                h.get_by_label(fix).click();
                h.run_steps(3);
            }
            assert!(
                h.state().analysis.errors.is_empty(),
                "{:?}",
                h.state().analysis.errors
            );
            assert!(
                h.state().s.code.contains("    ROTIRE\n")
                    && h.state().s.code.contains("EXECUTĂ Patrat")
            );
            shot(&mut h, "8-fixed");
        }

        #[test]
        fn errors_and_dark_theme() {
            let mut h = harness();
            h.ctx.set_theme(ThemePreference::Dark);
            h.state_mut().s.code =
                "[\nPAS\nREPETĂ 3 ORI\n    PAS\n    SARI\nSFÎRȘITUL REPETĂRII\n]\n".into();
            h.run_steps(2);
            h.get_by_label("✔  Verifică").click();
            h.run_steps(3);
            assert!(matches!(h.state().status, Status::Checked(1, 0)));
            assert_eq!(h.state().analysis.errors[0].line, 5);
            shot(&mut h, "4-compile-error");

            h.state_mut().s.code = "[\nREPETĂ 30 ORI\n    PAS\nSFÎRȘITUL REPETĂRII\n]\n".into();
            h.state_mut().s.instant = true;
            h.get_by_label("▶  Execută").click();
            run_until_idle(&mut h);
            assert!(matches!(
                h.state().status,
                Status::Runtime(RuntimeError::OutOfField, 3)
            ));
            assert!(h.state().crashed);
            shot(&mut h, "5-runtime-error");
        }

        #[test]
        fn russian_spiral_step_by_step() {
            let mut h = harness();
            h.ctx.set_theme(ThemePreference::Dark);
            h.state_mut().s.lang = Lang::Ru;
            h.state_mut().s.code = EXAMPLES[4].1.into();
            h.state_mut().s.show_jumps = true;
            h.run_steps(2);
            for _ in 0..60 {
                h.get_by_label("⏭  По шагам").click();
                h.run_steps(2);
            }
            assert_eq!(h.state().mode, Mode::Paused);
            assert_eq!(h.state().world.step_count, 60);
            h.get_by_label("❓  Помощь").click();
            h.run_steps(3);
            shot(&mut h, "6-russian-help");
        }
    }
}
