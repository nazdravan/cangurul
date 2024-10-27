// Pune iconița cangurului și descrierea în executabilul de Windows.
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/cangurul.ico");
    res.set("FileDescription", "Cangurul");
    res.set("ProductName", "Cangurul");

    // La compilarea din Linux pentru ținta win7, winresource nu ghicește prefixul uneltelor MinGW.
    let target = std::env::var("TARGET").unwrap_or_default();
    if cfg!(not(windows)) && target.ends_with("windows-gnu") {
        let prefix = if target.starts_with("i686") { "i686-w64-mingw32" } else { "x86_64-w64-mingw32" };
        res.set_windres_path(&format!("{prefix}-windres"));
        res.set_ar_path(&format!("{prefix}-ar"));
    }
    if let Err(e) = res.compile() {
        println!("cargo:warning=nu am putut adăuga iconița: {e}");
    }
}
