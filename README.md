<div align="center">

<p align="center">
  <img src="assets/logo.png" alt="Cangurul" width="130">
</p>

<img src="docs/assets/cangurul-banner.png" alt="Cangurul" />


**Mediu de învățare a algoritmilor pentru elevi — programul scris cu comenzi simple îl face pe cangur să sară pe câmp și să deseneze linii.**

[![Licență](https://img.shields.io/badge/licen%C8%9B%C4%83-GPL--3.0-3b82f6?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024-B7410E?style=flat-square&logo=rust&logoColor=white)](Cargo.toml)
[![egui](https://img.shields.io/badge/UI-egui%20%2F%20eframe-EB8137?style=flat-square)](https://github.com/emilk/egui)
[![Windows](https://img.shields.io/badge/Windows-7%20%7C%2010%20%7C%2011-0078D6?style=flat-square&logo=windows&logoColor=white)](#windows-7-32-și-64-de-biți)
[![Linux](https://img.shields.io/badge/Linux-x86__64-FCC624?style=flat-square&logo=linux&logoColor=black)](packaging/linux/)

[**Pornire rapidă**](#pornire-rapidă) · [Limbajul](#limbajul) · [Analizorul de cod](#analizorul-de-cod) · [Releases](../../releases) · [Issues](../../issues)

</div>

<br />

Rescris în **Rust** după programul „Cangurul” (versiunea 2).
Fișierele `.cng` din versiunea veche se deschid și rulează fără modificări.

<img src="docs/assets/screenshot.png" alt="Cangurul: un exemplu original după execuție" />

## Pornire rapidă

Arhivele gata construite sunt pe pagina [Releases](../../releases):

```
cangurul-<versiune>-linux-x86_64.tar.gz           # Linux
cangurul-<versiune>-windows-x64-portable.zip      # Windows 10/11, 64 de biți
cangurul-<versiune>-windows-x86-portable.zip      # Windows 10/11, 32 de biți
cangurul-<versiune>-windows7-x86-portable.zip     # Windows 7, 8, 10, 11 (32 și 64 de biți)
SHA256SUMS.txt                                    # sumele de control
```

Nu e nevoie de instalare: se dezarhivează și se pornește `Cangurul.exe`
(pe Linux, `cangurul`).

### Din cod sursă

Trebuie instalat [Rust](https://rustup.rs). Apoi, în acest dosar:

```sh
cargo run --release                 # pornește aplicația
cargo run --release -- program.cng  # deschide direct un fișier
```

Executabilul rezultat este în `target/release/` (`cangurul.exe` pe Windows) și
poate fi copiat pe alte calculatoare: nu are nevoie de alte fișiere. Pe Windows
iconița cangurului este inclusă automat în `.exe`. Pentru afișare se folosește
OpenGL 2.0, ca să meargă și pe calculatoarele mai vechi din laboratoare.

---

## Ce e nou față de versiunea originală

| | |
|---|---|
| **Interfață** | temă luminoasă și întunecată; interfața în română, rusă și engleză |
| **Editor** | numerotarea rândurilor, evidențierea sintaxei, butoane care inserează comenzile (PAS, SALT, REPETĂ, CÎT, DACĂ…) |
| **Execuție** | animație fluidă, rândul curent evidențiat în editor; pas cu pas, pauză, viteză reglabilă sau instant |
| **Analizor** | programul e verificat în timp ce scrii, cu sugestii de corectură ([detalii](#analizorul-de-cod)) |
| **Câmpul** | dimensiune reglabilă; salturile pot fi afișate punctat |
| **Limbi** | cuvintele-cheie pot fi scrise în română, rusă sau engleză, cu sau fără diacritice |
| **Fișiere** | programul și setările se păstrează între porniri; deschidere prin drag & drop sau din linia de comandă |

---

## Limbajul

| Comandă | Ce face |
|---|---|
| `PAS` | un pas înainte, lăsând urmă |
| `SALT` | un pas înainte, fără urmă |
| `ROTIRE` | rotire cu 90° în sensul acelor ceasornicului |
| `REPETĂ N ORI … SFÎRȘITUL REPETĂRII` | repetă de N ori |
| `CÎT [NU] condiție … SFÎRȘITUL CICLULUI` | repetă cât timp condiția e adevărată |
| `DACĂ [NU] condiție comandă1 ALTFEL comandă2` | ramificare (`ALTFEL` poate lipsi) |
| `PROCEDURA Nume … SFÎRȘITUL PROCEDURII` | definește o procedură (înainte de programul principal) |
| `EXECUTĂ Nume` | apelează procedura |
| `[ … ]` | programul principal |
| `// …` și `* … *` | comentarii |

Condiții: `E_MARGINE` (cangurul stă cu fața spre marginea câmpului) și
`E_LINIE` (în fața lui este deja o linie sau marginea).

Cangurul pornește din colțul din stânga-sus și privește spre dreapta.

### Cuvintele-cheie în trei limbi

| Română | English | Русский |
|---|---|---|
| `PAS SALT ROTIRE` | `STEP JUMP ROTATE` | `ШАГ ПРЫЖОК ПОВОРОТ` |
| `REPETĂ ORI` | `REPEAT TIMES` | `ПОВТОРИ РАЗ` |
| `CÎT` | `WHILE` | `ПОКА` |
| `DACĂ ATUNCI ALTFEL` | `IF THEN ELSE` | `ЕСЛИ ТОГДА ИНАЧЕ` |
| `NU` | `NOT` | `НЕ` |
| `PROCEDURA EXECUTĂ` | `PROCEDURE EXECUTE` | `ПРОЦЕДУРА ВЫПОЛНИТЬ` |
| `SFÎRȘITUL REPETĂRII / CICLULUI / PROCEDURII` | `END REPEAT / CYCLE / PROCEDURE` | `КОНЕЦ ПОВТОРА / ЦИКЛА / ПРОЦЕДУРЫ` |
| `E_MARGINE E_LINIE` | `IS_BORDER IS_LINE` | `КРАЙ ЛИНИЯ` |

### Taste rapide

| Tastă | Acțiune |
|---|---|
| **F5** | execută / pauză |
| **F6** | pas cu pas |
| **Esc** | stop |
| **F7** | verifică |
| **Ctrl+N / Ctrl+O / Ctrl+S** | nou / deschide / salvează |

---

## Analizorul de cod

<img src="docs/assets/analizor.png" alt="Analizorul de cod cu erori și sugestii" />

Programul este analizat la fiecare modificare (se poate opri din *Setări*;
atunci analiza se face cu **Verifică** / F7). Analizorul nu se oprește la prima
greșeală, ci le găsește pe toate:

- cuvintele greșite sunt subliniate ondulat (roșu = eroare, galben = avertisment),
  iar rândul are un marcaj lângă număr; la hover apare explicația;
- lista **Probleme** de sub editor arată toate erorile și avertismentele;
  un click duce cursorul la cuvântul respectiv;
- pentru cuvinte scrise greșit se propune corectura („Poate ați vrut SALT?”),
  aplicabilă cu butonul **Corectează** — și pentru nume de proceduri;
- programul cu erori nu pornește; avertismentele nu blochează rularea.

| Tip | Ce se detectează |
|---|---|
| **Erori** | cuvinte și condiții necunoscute; `REPETĂ` fără număr; blocuri neînchise sau închise greșit (`SFÎRȘITUL CICLULUI` în loc de `SFÎRȘITUL REPETĂRII`); `SFÎRȘITUL`/`ALTFEL`/`]` rătăcite; comandă lipsă după `DACĂ`; proceduri inexistente sau definite de două ori; proceduri scrise în interiorul `[ … ]`; text după programul principal; comentarii neînchise |
| **Avertismente** | proceduri nefolosite; blocuri goale; `REPETĂ 0 ORI`; cicluri `CÎT` care nu mută cangurul (pot rula la nesfârșit); proceduri care se apelează pe ele însele fără oprire; program principal gol |

### Decizii luate acolo unde comportamentul vechi nu era documentat

- câmpul implicit are 20 × 20 de celule (se poate schimba din *Setări*);
  toate exemplele originale încap în el;
- `E_LINIE` este adevărat și când în față este marginea câmpului — altfel
  exemplul original „Spirala” n-ar funcționa;
- după `DACĂ` și după `ALTFEL` urmează câte o singură comandă (poate fi și
  un `REPETĂ` sau `CÎT` întreg), ca în versiunea veche.

---

## Windows 7 (32 și 64 de biți)

Un `.exe` construit normal cere Windows 10+ (cerință a Rust de la versiunea
1.78). Pentru laboratoarele cu Windows 7 există o variantă separată, construită
din Linux cu:

```sh
packaging/windows7/package.sh   # rezultatul: dist/release/cangurul-<versiune>-windows7-x86-portable.zip
```

Scriptul instalează singur Rust nightly în `.cache/` și descarcă ce îi trebuie;
din sistem cere doar `curl`, `7z` și compilatorul MinGW pe 32 de biți
(`mingw-w64-gcc` pe Arch, `gcc-mingw-w64-i686` pe Debian/Ubuntu). Arhiva
(~23 MB) merge pe Windows 7, 8, 10 și 11, pe 32 și pe 64 de biți, fără
instalare și fără actualizări suplimentare de Windows:

- folosește `msvcrt.dll` (există pe orice Windows 7), nu UCRT, care pe
  Windows 7 cere actualizarea KB2999226;
- `combase.dll` din arhivă (3,5 KB) redirecționează o funcție pe care
  bibliotecile Rust noi o caută acolo, dar care pe Windows 7 este în
  `ole32.dll`; pe Windows 8+ e ignorat;
- dosarul `soft-gl` conține Mesa (OpenGL software). Dacă placa video nu are
  driver cu OpenGL 2.0 (driver „Standard VGA”, mașini virtuale), programul se
  repornește singur cu Mesa. Mai lent, dar funcționează. Se poate forța cu
  `Cangurul.exe --software-gl`;
- în această variantă lipsește doar suportul pentru cititoare de ecran
  (biblioteca respectivă cere Windows 8+). În cod e funcția `accessibility`,
  activă implicit în celelalte build-uri.

**Atenție:** varianta a fost verificată static (fiecare funcție importată din
Windows există în Windows 7), dar nu a fost încă pornită pe un Windows 7 real.

---

## Structura codului

| Fișier | Conținut |
|---|---|
| `src/lang.rs` | lexer, parser tolerant la erori și compilator |
| `src/check.rs` | analiza semantică: avertismente, proceduri, sugestii de corectură |
| `src/vm.rs` | mașina virtuală care execută câte o acțiune pe rând |
| `src/world.rs` | câmpul, cangurul, liniile |
| `src/app.rs` | interfața grafică (egui / eframe) |
| `src/i18n.rs` | textele interfeței în română, rusă și engleză |
| `src/software_gl.rs` | repornirea cu OpenGL software (Mesa) pe Windows |
| `examples/` | exemplele originale (convertite în UTF-8) |
| `assets/` | sprite-urile cangurului, logo-ul, fonturi; desenele se generează cu `assets/art/make.py`, bannerul cu `python3 assets/banner.py CANGURUL -o docs/assets/cangurul-banner.png --color "#eb8137"` |
| `packaging/` | arhivele pentru Linux, Windows 10/11 și Windows 7 |

---

## Release

Versiunea există într-un singur loc, `Cargo.toml`, și se schimbă cu
`packaging/version.sh set 3.1.0`. La un push pe ramura `release`, GitHub Actions
construiește arhivele pentru toate platformele și creează un **draft** de
release, care se publică manual. Pașii sunt descriși în
[`docs/releasing.md`](docs/releasing.md).

## Teste

```sh
cargo test
```

Testele verifică limbajul, rularea tuturor exemplelor originale și interfața:
apasă butoanele reale fără fereastră. Cu `CANGURUL_SHOTS=<dosar> cargo test`
se salvează și capturi de ecran (cele din `docs/assets/` sunt făcute așa).

## Licență

Cangurul 3 este software liber, distribuit sub licența
[GNU General Public License v3](LICENSE) sau, la alegere, orice versiune
ulterioară (`GPL-3.0-or-later`).
