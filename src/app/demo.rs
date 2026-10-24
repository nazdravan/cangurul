//! Înregistrarea demo-ului video: aplicația adevărată în harness, cu timp
//! simulat (un pas = un cadru), cadrele merg direct în ffmpeg.
//!
//! CANGURUL_DEMO=<dosar cu voice.json și voice.mp3> \
//!     cargo test --release demo -- --ignored --nocapture
//!
//! Fiecare scenă începe la începutul paragrafului ei din voice.json, ca
//! acțiunea să cadă pe vocea care o descrie. Rezultatul: <dosar>/video.mp4.

use super::*;
use egui::{Event, PointerButton};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use std::io::Write;
use std::process::{Child, Command, Stdio};

const FPS: f64 = 30.0;
const SIZE: Vec2 = vec2(1280.0, 720.0);
const PPP: f32 = 1.5;
/// Înălțimea unui rând în editor, în puncte.
const LINE_H: f32 = 18.67;

struct Rec<'a> {
    h: Harness<'a, CangurApp>,
    ff: Child,
    frame: u64,
    mouse: Pos2,
    paras: Vec<(String, f64)>,
    duration: f64,
    /// Eticheta cu linkul (link.png din dosarul demo-ului) și de când apare.
    link: Option<(image::RgbaImage, f64)>,
}

impl Rec<'_> {
    fn now(&self) -> f64 {
        self.frame as f64 / FPS
    }

    /// Un cadru: un pas al aplicației și imaginea lui spre ffmpeg.
    fn tick(&mut self) {
        self.h.step();
        let mut img = self.h.render().unwrap();
        if let Some((link, from)) = &self.link {
            let fade = ((self.now() - from) / 0.4).clamp(0.0, 1.0) as f32;
            if fade > 0.0 {
                blend(&mut img, link, fade);
            }
        }
        self.ff
            .stdin
            .as_mut()
            .unwrap()
            .write_all(img.as_raw())
            .unwrap();
        self.frame += 1;
    }

    /// Captură PNG pentru depanarea scenariului, dacă `DEMO_SHOTS` e setat.
    fn shot(&mut self, name: &str) {
        if let Ok(dir) = std::env::var("DEMO_SHOTS") {
            self.h
                .render()
                .unwrap()
                .save(format!("{dir}/{name}.png"))
                .unwrap();
        }
    }

    fn event(&mut self, e: Event) {
        self.h.input_mut().events.push(e);
    }

    fn wait(&mut self, secs: f64) {
        for _ in 0..(secs * FPS).round() as u64 {
            self.tick();
        }
    }

    /// Așteaptă începutul paragrafului `id` din voce, plus `delay` secunde.
    fn at(&mut self, id: &str, delay: f64) {
        let start = self
            .paras
            .iter()
            .find(|(p, _)| p == id)
            .unwrap_or_else(|| panic!("nu e paragraful {id} în voice.json"))
            .1;
        while self.now() < start + delay {
            self.tick();
        }
    }

    /// Mută mausul lin până la `to`.
    fn move_to(&mut self, to: Pos2) {
        let from = self.mouse;
        let n = ((from.distance(to) / 900.0 * FPS as f32).clamp(8.0, 22.0)) as u32;
        for i in 1..=n {
            let k = i as f32 / n as f32;
            let k = k * k * (3.0 - 2.0 * k);
            self.mouse = from + (to - from) * k;
            self.event(Event::PointerMoved(self.mouse));
            self.tick();
        }
    }

    fn press(&mut self, pressed: bool) {
        self.event(Event::PointerButton {
            pos: self.mouse,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::default(),
        });
        self.tick();
    }

    fn click_at(&mut self, to: Pos2) {
        self.move_to(to);
        self.wait(0.15);
        self.press(true);
        self.press(false);
        self.wait(0.25);
    }

    /// Click pe butonul cu exact acest text.
    fn click(&mut self, label: &str) {
        let at = self.h.get_by_label(label).rect().center();
        self.click_at(at);
    }

    /// Click pe butonul al cărui text conține `part` (butoanele au iconițe).
    fn click_on(&mut self, part: &str) {
        let at = self.h.get_by_label_contains(part).rect().center();
        self.click_at(at);
    }

    /// Mausul pe butonul cu exact acest text, fără click: butonul se aprinde cât timp vocea îl numește.
    fn hover(&mut self, label: &str) {
        let at = self.h.get_by_label(label).rect().center();
        self.move_to(at);
    }

    /// Trage glisorul Viteza până la valoarea `v` (1–10).
    fn drag_speed(&mut self, v: f32) {
        let rect = self.h.get_by_role(egui::accesskit::Role::Slider).rect();
        let r = rect.height() / 2.5;
        let x = |v: f32| rect.left() + r + (v - 1.0) / 9.0 * (rect.width() - 2.0 * r);
        let from = self.h.state().s.speed;
        self.move_to(pos2(x(from), rect.center().y));
        self.press(true);
        self.move_to(pos2(x(v), rect.center().y));
        self.press(false);
        self.h.state_mut().s.speed = v;
        self.wait(0.3);
    }

    fn key(&mut self, key: Key) {
        for pressed in [true, false] {
            self.event(Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: Modifiers::default(),
            });
        }
        self.tick();
        self.wait(0.12);
    }

    /// Tasta cu Ctrl (Cmd pe macOS), de exemplu Ctrl+A.
    fn command_key(&mut self, key: Key) {
        for pressed in [true, false] {
            self.event(Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: Modifiers::COMMAND,
            });
        }
        self.tick();
        self.wait(0.12);
    }

    fn type_text(&mut self, text: &str) {
        for c in text.chars() {
            self.event(Event::Text(c.to_string()));
            self.tick();
            self.wait(0.1);
        }
    }

    fn idle(&self) -> bool {
        self.h.state().mode == Mode::Idle && self.h.state().anim.is_none()
    }

    fn until_idle(&mut self) {
        while !self.idle() {
            self.tick();
        }
    }
}

/// Pune eticheta jos, la centru, cu opacitatea `fade`.
fn blend(img: &mut image::RgbaImage, link: &image::RgbaImage, fade: f32) {
    let x0 = (img.width() - link.width()) / 2;
    let y0 = img.height() - link.height() - img.height() / 7;
    for (x, y, p) in link.enumerate_pixels() {
        let a = p[3] as f32 / 255.0 * fade;
        let q = img.get_pixel_mut(x0 + x, y0 + y);
        for c in 0..3 {
            q[c] = (p[c] as f32 * a + q[c] as f32 * (1.0 - a)).round() as u8;
        }
    }
}

/// Paragrafele vocii (id, început) și durata totală, din voice.json.
fn read_voice(json: &str) -> (Vec<(String, f64)>, f64) {
    let num = |s: &str| -> f64 {
        let s = s.trim_start();
        let end = s
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap_or(s.len());
        s[..end].parse().unwrap()
    };
    let duration = num(json.split("\"duration\":").nth(1).unwrap());
    let paras = json
        .split("\"id\": \"")
        .skip(1)
        .map(|p| {
            let id = p[..p.find('"').unwrap()].to_string();
            (id, num(p.split("\"start\":").nth(1).unwrap()))
        })
        .collect();
    (paras, duration)
}

fn scenario(r: &mut Rec<'_>) {
    // Ce este: exemplul „Cangur” scrie CANGUR pe câmp.
    r.at("p01", 0.6);
    r.click_on("Exemple");
    r.click("Cangur");
    r.wait(0.5);
    r.click_on("Execută");

    // Primul program: pătratul, numai din butoane.
    r.at("p03", 0.3);
    r.until_idle();
    r.click_on("Nou");
    // Comenzile, arătate cât sunt numite; momentele sunt ale cuvintelor din voce (Whisper pe voice.mp3).
    for (b, t) in [("PAS", 2.4), ("SALT", 7.1), ("ROTIRE", 10.0)] {
        r.at("p03", t);
        r.hover(b);
    }
    r.at("p04", 3.2);
    for b in ["REPETĂ N", "PAS", "PAS", "PAS", "ROTIRE"] {
        r.click(b);
        r.wait(0.6);
    }

    // Execuția: viteza mai mică, ca să se vadă rândul curent.
    r.at("p05", 0.0);
    r.drag_speed(3.0);
    r.click_on("Execută");
    r.at("p06", 0.3);
    r.until_idle();
    r.drag_speed(6.0);
    r.at("p06", 3.0);
    for _ in 0..5 {
        r.click_on("Pas cu pas");
        r.wait(0.7);
    }
    r.at("p06", 11.0);
    r.click_on("Instant");
    r.click_on("Execută");
    r.wait(1.0);
    r.click_on("Instant");

    // Analizorul: ROTIRE devine ROTIRA, apoi „Corectează”.
    r.at("p07", 0.3);
    // Click la dreapta de textul rândului cu ROTIRE: cursorul ajunge la capătul lui.
    let editor =
        r.h.get_by_role(egui::accesskit::Role::MultilineTextInput)
            .rect();
    let row =
        r.h.state()
            .s
            .code
            .lines()
            .position(|l| l.contains("ROTIRE"))
            .unwrap();
    r.click_at(pos2(
        editor.left() + 180.0,
        editor.top() + 6.0 + LINE_H * (row as f32 + 0.5),
    ));
    r.wait(0.5);
    r.key(Key::Backspace);
    r.type_text("A");
    r.wait(0.5);
    r.shot("typo");
    r.at("p08", 1.8);
    let fix = r.h.get_by_label_contains("Corectează").rect().center();
    r.move_to(fix);
    r.at("p08", 4.2);
    r.click_on("Corectează");
    r.wait(0.8);
    r.click_on("Execută");

    // Exemple și limbi: spirala în română, apoi în engleză, apoi tema întunecată.
    r.at("p09", 0.0);
    r.until_idle();
    r.h.state_mut().s.dirty = false;
    r.click_on("Exemple");
    r.click("Spirala (RO)");
    r.drag_speed(9.0);
    r.click_on("Execută");
    r.at("p09", 6.4);
    r.click_on("Exemple");
    r.click("Spiral (EN)");
    r.click_on("Instant");
    r.click_on("Execută");
    // „interfața are aceleași trei limbi și o temă întunecată”
    r.at("p09", 11.4);
    r.click_on("Română");
    r.click("English");
    r.click_on("Settings");
    r.click_on("Dark");
    r.key(Key::Escape);

    r.move_to(pos2(SIZE.x - 40.0, SIZE.y - 60.0));
    while r.now() < r.duration + 3.0 {
        r.tick();
    }
}

/// Programul „copiat din altă parte”: corect, dar fără indentare.
const PASTED: &str = "PROCEDURA Linie\nCÎT NU E_MARGINE\nPAS\nSFÎRȘITUL CICLULUI\nSFÎRȘITUL PROCEDURII\n[\nREPETĂ 4 ORI\nEXECUTĂ Linie\nDACĂ E_MARGINE ROTIRE ALTFEL SALT\nSFÎRȘITUL REPETĂRII\n]\n";

/// Programul lipit în demo se compilează, iar Formatează chiar îl schimbă.
#[test]
fn pasted_program() {
    assert!(crate::lang::compile(PASTED).is_ok());
    assert_ne!(crate::lang::format_program(PASTED), PASTED);
}

#[test]
#[ignore = "înregistrează demo-ul video; vezi antetul fișierului"]
fn demo() {
    let dir = std::path::PathBuf::from(
        std::env::var("CANGURUL_DEMO").expect("CANGURUL_DEMO = dosarul cu voice.json"),
    );
    let (paras, duration) = read_voice(&std::fs::read_to_string(dir.join("voice.json")).unwrap());

    let mut h = Harness::builder()
        .with_size(SIZE)
        .with_pixels_per_point(PPP)
        .with_step_dt((1.0 / FPS) as f32)
        .wgpu()
        .build_eframe(|cc| CangurApp::new(cc, None));
    h.ctx.set_theme(ThemePreference::Light);
    h.state_mut().s.theme = ThemeChoice::Light;
    h.state_mut().s.speed = 8.0;
    h.state_mut().s.instant = false;
    h.run_steps(3);

    let (w, hgt) = ((SIZE.x * PPP) as u32, (SIZE.y * PPP) as u32);
    let ff = Command::new("ffmpeg")
        .args([
            "-y",
            "-loglevel",
            "error",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgba",
        ])
        .args([
            "-s",
            &format!("{w}x{hgt}"),
            "-r",
            &FPS.to_string(),
            "-i",
            "-",
        ])
        .arg("-i")
        .arg(dir.join("voice.mp3"))
        .args([
            "-c:v", "libx264", "-preset", "medium", "-crf", "18", "-pix_fmt", "yuv420p",
        ])
        .args(["-c:a", "aac", "-b:a", "160k", "-movflags", "+faststart"])
        .arg(dir.join("video.mp4"))
        .stdin(Stdio::piped())
        .spawn()
        .expect("ffmpeg");

    let mut r = Rec {
        h,
        ff,
        frame: 0,
        mouse: pos2(SIZE.x * 0.6, SIZE.y * 0.6),
        link: image::open(dir.join("link.png"))
            .ok()
            .map(|i| (i.to_rgba8(), paras.last().unwrap().1)),
        paras,
        duration,
    };
    scenario(&mut r);
    drop(r.ff.stdin.take());
    assert!(r.ff.wait().unwrap().success());
}
