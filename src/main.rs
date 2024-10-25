#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod check;
mod i18n;
mod lang;
#[cfg(windows)]
mod software_gl;
mod vm;
mod world;

use std::path::PathBuf;

fn main() -> eframe::Result {
    #[cfg(windows)]
    let software = software_gl::enable_if_requested();

    // Un fișier .cng poate fi dat ca argument (ex. deschis din Explorer).
    let open = std::env::args_os()
        .skip(1)
        .find(|a| !a.to_string_lossy().starts_with("--"))
        .map(PathBuf::from);

    let icon = app::decode_png(include_bytes!("../assets/logo.png"));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Cangurul")
            .with_app_id("cangurul")
            .with_inner_size([1180.0, 740.0])
            .with_min_inner_size([800.0, 520.0])
            // Ecranele calculatoarelor vechi sunt mici (1024×768, 1366×768).
            .with_maximized(true)
            .with_icon(egui::IconData {
                rgba: icon.pixels.iter().flat_map(|c| c.to_srgba_unmultiplied()).collect(),
                width: icon.size[0] as u32,
                height: icon.size[1] as u32,
            }),
        ..Default::default()
    };
    let result = eframe::run_native(
        "Cangurul",
        options,
        Box::new(|cc| Ok(Box::new(app::CangurApp::new(cc, open)))),
    );

    if let Err(e) = &result {
        // Fără placă video compatibilă: încearcă din nou cu OpenGL software (Mesa).
        #[cfg(windows)]
        if !software && software_gl::relaunch() {
            return Ok(());
        }
        // Fereastra nu a putut fi deschisă: explicăm de ce, altfel elevul n-ar vedea nimic.
        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("Cangurul")
            .set_description(format!(
                "Cangurul nu poate porni: placa video nu suportă OpenGL 2.0.\n\
                 Instalați driverul plăcii video sau puneți dosarul „soft-gl” lângă program.\n\n{e}"
            ))
            .show();
    }
    result
}
