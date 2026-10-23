# Changelog

[Română](CHANGELOG.md) · **English** · [Русский](CHANGELOG.ru.md)

The notable changes in each version. Versions follow
[Semantic Versioning](https://semver.org/).

## [3.0.3] — 2026-10-23

### Added

- Windows `.msi` installer: the install folder can be changed, and a
  checkbox (ticked by default) creates a Cangurul shortcut on the desktop.
  Unattended: `msiexec /i cangurul-*.msi /qn INSTALLFOLDER="D:\Cangurul"
  DESKTOPSHORTCUT=""`.

### Changed

- The `.msi` installer windows are in Romanian, without the license page.
- A new version installs into the folder chosen during the previous install.

## [3.0.2] — 2026-10-22

### Added

- The **Format** button (Ctrl+Shift+F): re-indents the program and puts the
  command after `IF` and `ELSE` on lines of their own. The cursor stays next
  to the same word, and Ctrl+Z undoes the formatting in one go.
- Suggestions while typing: after `wh` you get `WHILE NOT IS_BORDER …`,
  `WHILE IS_BORDER …` and the other conditions; `EXECUTE` suggests the
  procedures in the program, and `END` the end of the open block. Pick one
  with ↑/↓ and Enter or Tab, or with the mouse; Esc closes the list.
- Enter keeps the block's indentation; `END` and `ELSE` line up with the start
  of their block by themselves.

### Changed

- Command buttons: the command gets the indentation of the block it lands in,
  the example inside it (`STEP`, `ROTATE`, `Name`) stays selected so that the
  next command replaces it, and after a simple command the cursor moves to a
  new line.
- `IF` is inserted on several lines, together with `ELSE`; after the command
  under `IF`, the one under `ELSE` gets selected.
- The examples are formatted.

### Fixed

- Copy and paste (Ctrl+C, Ctrl+X, Ctrl+V) work with other programs too.
- The cursor no longer disappears from the editor after clicking the command
  buttons or **Fix**.
- Clicking in the problem list keeps the misspelled word selected.

## [3.0.1] — 2026-10-22

### Added

- An `.msi` installer for 64- and 32-bit Windows 10/11: Start menu entry,
  `.cng` file association, upgrades over the old version.
- README in English and Russian.

### Changed

- The command buttons are on two rows: the actions first, then the control
  structures.

### Fixed

- The portable Windows archive: license texts are read correctly (UTF-8).

## [3.0.0] — 2026-10-20

The first version rewritten in Rust, based on the “Cangurul” program
(version 2). `.cng` files from the old version open and run unchanged.

- Interface in Romanian, Russian and English, with a light and a dark theme.
- An editor with line numbers, syntax highlighting and buttons that insert
  commands.
- Animated execution with the current line highlighted; step by step, pause,
  adjustable speed or instant.
- The analyzer checks the program as you type and suggests fixes.
- Adjustable field size; jumps can be shown as dotted lines.
- Keywords can be written in Romanian, Russian or English, with or without
  diacritics.
- Packages for Windows (including Windows 7), Linux and macOS.

[3.0.3]: https://github.com/nazdravan/cangurul/compare/v3.0.2...v3.0.3
[3.0.2]: https://github.com/nazdravan/cangurul/compare/v3.0.1...v3.0.2
[3.0.1]: https://github.com/nazdravan/cangurul/compare/v3.0.0...v3.0.1
[3.0.0]: https://github.com/nazdravan/cangurul/releases/tag/v3.0.0
