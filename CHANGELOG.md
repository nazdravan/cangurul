# Jurnalul modificărilor

**Română** · [English](CHANGELOG.en.md) · [Русский](CHANGELOG.ru.md)

Modificările importante din fiecare versiune. Numerotarea urmează
[Semantic Versioning](https://semver.org/lang/ro/).

## [3.0.5] — 2026-10-24

### Reparat

- Linkurile din fereastra **Ajutor** se deschid în browser (adresa de email,
  în programul de email).

## [3.0.4] — 2026-10-24

### Reparat

- Cangurul din rândul de sus al câmpului se vede întreg: pe ecranele mari
  (de exemplu pe Mac) urechile lui erau tăiate.
- Meniul **Setări** rămâne deschis cât timp bifezi opțiunile și se închide
  la un click în afara lui.

## [3.0.3] — 2026-10-23

### Adăugat

- Instalatorul `.msi` pentru Windows: dosarul de instalare se poate schimba,
  iar o bifă (implicit bifată) creează scurtătura Cangurul pe desktop. Fără
  ferestre: `msiexec /i cangurul-*.msi /qn INSTALLFOLDER="D:\Cangurul"
  DESKTOPSHORTCUT=""`.

### Schimbat

- Ferestrele instalatorului `.msi` sunt în română, fără pagina cu licența.
- O versiune nouă se instalează în dosarul ales la instalarea precedentă.

## [3.0.2] — 2026-10-22

### Adăugat

- Butonul **Formatează** (Ctrl+Shift+F): reindentează programul și pune pe
  rânduri separate comanda de după `DACĂ` și `ALTFEL`. Cursorul rămâne lângă
  același cuvânt, iar Ctrl+Z anulează formatarea dintr-o dată.
- Variante propuse în timp ce scrii: după `cit` apar `CÎT NU E_MARGINE …`,
  `CÎT E_MARGINE …` și celelalte condiții; `EXECUTĂ` propune procedurile din
  program, iar `SFÎRȘITUL` — sfârșitul blocului deschis. Se aleg cu ↑/↓ și
  Enter sau Tab, ori cu mausul; Esc închide lista.
- Enter păstrează indentarea blocului; `SFÎRȘITUL` și `ALTFEL` se aliniază
  singure cu începutul blocului.

### Schimbat

- Butoanele de comenzi: comanda primește indentarea blocului în care ajunge,
  exemplul din interior (`PAS`, `ROTIRE`, `Nume`) rămâne selectat, ca
  următoarea comandă să-l înlocuiască, iar după o comandă simplă cursorul
  trece pe un rând nou.
- `DACĂ` se inserează pe mai multe rânduri, împreună cu `ALTFEL`; după
  comanda de sub `DACĂ` se selectează cea de sub `ALTFEL`.
- Exemplele sunt formatate.

### Reparat

- Copierea și lipirea (Ctrl+C, Ctrl+X, Ctrl+V) funcționează și cu alte
  programe.
- Cursorul nu mai dispare din editor după click pe butoanele de comenzi și pe
  **Corectează**.
- La click în lista de probleme, cuvântul greșit rămâne selectat.

## [3.0.1] — 2026-10-22

### Adăugat

- Instalator `.msi` pentru Windows 10/11 pe 64 și pe 32 de biți: meniul
  Start, asocierea fișierelor `.cng`, actualizare peste versiunea veche.
- README în engleză și în rusă.

### Schimbat

- Butoanele de comenzi sunt pe două rânduri: întâi acțiunile, apoi
  structurile de control.

### Reparat

- Arhiva portabilă pentru Windows: textele licențelor se citesc corect
  (UTF-8).

## [3.0.0] — 2026-10-20

Prima versiune rescrisă în Rust după programul „Cangurul” (versiunea 2).
Fișierele `.cng` din versiunea veche se deschid și rulează fără modificări.

- Interfața în română, rusă și engleză, cu temă luminoasă și întunecată.
- Editor cu numerotarea rândurilor, evidențierea sintaxei și butoane care
  inserează comenzile.
- Execuție animată, cu rândul curent evidențiat; pas cu pas, pauză, viteză
  reglabilă sau instant.
- Analizorul verifică programul în timp ce scrii și propune corecturi.
- Câmpul are dimensiune reglabilă; salturile pot fi afișate punctat.
- Cuvintele-cheie se pot scrie în română, rusă sau engleză, cu sau fără
  diacritice.
- Pachete pentru Windows (inclusiv Windows 7), Linux și macOS.

[3.0.3]: https://github.com/nazdravan/cangurul/compare/v3.0.2...v3.0.3
[3.0.2]: https://github.com/nazdravan/cangurul/compare/v3.0.1...v3.0.2
[3.0.1]: https://github.com/nazdravan/cangurul/compare/v3.0.0...v3.0.1
[3.0.0]: https://github.com/nazdravan/cangurul/releases/tag/v3.0.0
