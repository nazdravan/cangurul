//! OpenGL software (Mesa llvmpipe) pentru calculatoarele fără driver video.
//!
//! Dacă lângă program există dosarul `soft-gl` cu `opengl32.dll` și
//! `libgallium_wgl.dll` din Mesa, iar OpenGL-ul sistemului nu merge (driver
//! „Standard VGA”, Windows 7 fără driver, mașini virtuale), programul se
//! repornește singur folosind Mesa. Pe calculatoarele cu driver bun Mesa nu
//! este folosit, pentru că e mai lent.

use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::process::Command;

/// Pornește direct în modul software (pus de `relaunch` sau de profesor).
const ENV: &str = "CANGURUL_SOFTWARE_GL";
const ARG: &str = "--software-gl";

#[link(name = "kernel32")]
unsafe extern "system" {
    fn SetDllDirectoryW(path: *const u16) -> i32;
}

fn mesa_dir() -> Option<PathBuf> {
    let dir = std::env::current_exe().ok()?.parent()?.join("soft-gl");
    dir.join("opengl32.dll").is_file().then_some(dir)
}

/// Activează Mesa dacă s-a cerut; întoarce `true` dacă modul software e activ.
pub fn enable_if_requested() -> bool {
    let requested = std::env::var_os(ENV).is_some() || std::env::args().any(|a| a == ARG);
    let Some(dir) = mesa_dir().filter(|_| requested) else {
        return false;
    };
    let wide: Vec<u16> = dir.as_os_str().encode_wide().chain(Some(0)).collect();
    // SAFETY: șir terminat cu zero, valabil pe durata apelului; încă rulează un singur fir.
    unsafe {
        // `libgallium_wgl.dll` se caută și în dosarul `soft-gl`.
        SetDllDirectoryW(wide.as_ptr());
        // glutin încarcă DLL-ul OpenGL de la această cale în locul celui din sistem.
        std::env::set_var("GLUTIN_WGL_OPENGL_DLL", dir.join("opengl32.dll"));
        std::env::set_var("GALLIUM_DRIVER", "llvmpipe");
    }
    true
}

/// Repornește programul în modul software; `false` dacă Mesa lipsește.
pub fn relaunch() -> bool {
    if mesa_dir().is_none() {
        return false;
    }
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    Command::new(exe).args(std::env::args_os().skip(1)).env(ENV, "1").spawn().is_ok()
}
