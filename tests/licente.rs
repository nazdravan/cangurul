//! Fiecare font inclus are licența lui lângă el, cu notița de copyright a fontului pe prima linie,
//! și e trecut în NOTICE; altfel un font nou ar ajunge în pachete fără licență.

use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

/// Dosarele cu fonturi care ajung la utilizatori: programul și pagina de prezentare.
const FONT_DIRS: [&str; 2] = ["assets/fonts", "site/assets"];

fn fonts() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for dir in FONT_DIRS {
        for entry in fs::read_dir(Path::new(ROOT).join(dir)).unwrap() {
            let path = entry.unwrap().path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if matches!(ext, "ttf" | "otf" | "woff2") {
                out.push(path);
            }
        }
    }
    assert!(!out.is_empty());
    out
}

/// `NotoSans-Bold.ttf` -> `OFL-NotoSans.txt`, în același dosar.
fn licence_of(font: &Path) -> PathBuf {
    let stem = font.file_stem().unwrap().to_str().unwrap();
    let family = stem.split('-').next().unwrap();
    font.with_file_name(format!("OFL-{family}.txt"))
}

/// Notița de copyright a fontului (tabela `name`, ID 0), din TrueType/OpenType.
fn copyright(data: &[u8]) -> Option<String> {
    let u16_at = |o: usize| Some(u16::from_be_bytes(data.get(o..o + 2)?.try_into().ok()?));
    let u32_at = |o: usize| Some(u32::from_be_bytes(data.get(o..o + 4)?.try_into().ok()?));
    let tables = u16_at(4)? as usize;
    let name = (0..tables).find_map(|i| {
        let rec = 12 + 16 * i;
        (data.get(rec..rec + 4)? == b"name").then(|| u32_at(rec + 8))?
    })? as usize;
    let count = u16_at(name + 2)? as usize;
    let strings = name + u16_at(name + 4)? as usize;
    (0..count).find_map(|i| {
        let rec = name + 6 + 12 * i;
        let (platform, id) = (u16_at(rec)?, u16_at(rec + 6)?);
        if id != 0 || platform != 3 {
            return None;
        }
        let (len, off) = (u16_at(rec + 8)? as usize, u16_at(rec + 10)? as usize);
        let raw = data.get(strings + off..strings + off + len)?;
        let units: Vec<u16> = raw
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16(&units).ok()
    })
}

#[test]
fn fiecare_font_are_licenta_cu_notita_lui() {
    for font in fonts() {
        let licence = licence_of(&font);
        let text = fs::read_to_string(&licence)
            .unwrap_or_else(|_| panic!("{} nu are {}", font.display(), licence.display()));
        let first = text.lines().next().unwrap_or("");
        assert!(
            text.contains("SIL OPEN FONT LICENSE Version 1.1"),
            "{}",
            licence.display()
        );
        // woff2 e comprimat: notița se verifică pe fontul .ttf din care e făcut.
        if font.extension().is_some_and(|e| e != "woff2") {
            let notice = copyright(&fs::read(&font).unwrap()).unwrap();
            assert!(
                notice.starts_with(first),
                "prima linie din {} trebuie să fie notița fontului {}: {notice}",
                licence.display(),
                font.display()
            );
        }
    }
}

#[test]
fn notice_numeste_fiecare_licenta() {
    let notice = fs::read_to_string(Path::new(ROOT).join("NOTICE")).unwrap();
    for font in fonts() {
        let licence = licence_of(&font);
        let name = licence.file_name().unwrap().to_str().unwrap();
        assert!(notice.contains(name), "NOTICE nu pomenește {name}");
    }
    for entry in fs::read_dir(Path::new(ROOT).join("licenses")).unwrap() {
        let dir = entry.unwrap().file_name();
        let dir = format!("licenses/{}/", dir.to_str().unwrap());
        assert!(notice.contains(&dir), "NOTICE nu pomenește {dir}");
    }
}

#[test]
fn licenta_mesa_e_pentru_versiunea_din_pachet() {
    let script = fs::read_to_string(Path::new(ROOT).join("packaging/windows7/package.sh")).unwrap();
    let version = script
        .lines()
        .find_map(|l| l.strip_prefix("MESA_VERSION="))
        .unwrap();
    let readme = fs::read_to_string(Path::new(ROOT).join("licenses/mesa/README.txt")).unwrap();
    let notice = fs::read_to_string(Path::new(ROOT).join("NOTICE")).unwrap();
    assert!(
        readme.starts_with(&format!("Mesa 3D {version} ")),
        "licenses/mesa/README.txt"
    );
    assert!(notice.contains(&format!("Mesa 3D {version} ")), "NOTICE");
}
