# Cum se face un release

Release-urile se fac **automat** pe GitHub Actions (`.github/workflows/release.yml`)
la fiecare push pe `main`:

- se rulează testele;
- dacă versiunea din `Cargo.toml` **nu e încă publicată**, se construiesc
  pachetele pentru toate platformele și se publică release-ul „Cangurul
  v<versiune>”, cu tag-ul `v<versiune>`, fișierul `SHA256SUMS.txt` și notele
  versiunii din `CHANGELOG.md`, urmate de lista commit-urilor generată de GitHub. Release-ul e creat întâi ca draft, i se urcă toate
  fișierele și abia apoi devine public;
- dacă versiunea e deja publicată, nu se construiește nimic în plus: un push
  obișnuit nu face release. Un release publicat nu e niciodată suprascris;
- dacă un build sau testele eșuează, nu se publică nimic;
- o versiune cu sufix (`3.1.0-rc.1`) este publicată ca *pre-release*.

## Versiunea

Versiunea există într-un singur loc: `version` din secțiunea `[package]` a
fișierului `Cargo.toml`.

```sh
packaging/version.sh              # afișează versiunea
packaging/version.sh set 3.1.0    # o schimbă în Cargo.toml și Cargo.lock
```

## Jurnalul modificărilor

Fiecare versiune are o secțiune `## [3.1.0] — <data>` în `CHANGELOG.md`
(română), `CHANGELOG.en.md` și `CHANGELOG.ru.md`, cu subtitlurile *Adăugat*,
*Schimbat*, *Reparat*. Secțiunea din `CHANGELOG.md` devine nota release-ului;
fără ea workflow-ul se oprește înainte de build-uri.

```sh
packaging/changelog.sh            # notele versiunii din Cargo.toml
packaging/changelog.sh 3.1.0      # notele altei versiuni
```

## Pași

1. Pe `main`, mărește versiunea, scrie secțiunea ei în cele trei jurnale de
   modificări, fă commit și push:
   ```sh
   packaging/version.sh set 3.1.0
   git add CHANGELOG*.md
   git commit -am "Versiunea 3.1.0"
   git push
   ```
2. În tab-ul *Actions* urmărește workflow-ul **Release** (cel mai lung e
   build-ul pentru Windows 7, ~20–30 de minute). La final release-ul apare
   publicat în *Releases*.

Dacă un build a eșuat, corectează și fă din nou push pe `main`, cu aceeași
versiune: cât timp versiunea nu e publicată, fiecare push reîncearcă release-ul.
Dacă release-ul publicat are o greșeală, se face o versiune nouă (`3.1.1`).

Commit-ul din care se face release-ul nu trebuie să modifice fișierele din
`.github/workflows/`: tokenul workflow-ului nu are voie să creeze tag-uri pe
astfel de commit-uri, iar publicarea pică cu `HTTP 403: Resource not accessible
by integration` (după ce toate build-urile au reușit). Schimbările de workflow
se urcă într-un commit separat, înaintea celui cu versiunea; dacă s-a întâmplat
deja, orice commit nou care nu atinge workflow-ul reîncearcă release-ul.

## Rulare de test

Din *Actions → Release → Run workflow* se poate porni workflow-ul pe orice
ramură, opțional cu o versiune de test (de ex. `3.1.0-rc.1`). Se fac doar
testele și build-urile, iar pachetele rămân ca artefacte ale rulării. Nu se
creează niciun release. Așa se pot încerca pachetele înainte de a mări versiunea.

## Pagina programului

<https://nazdravan.github.io/cangurul/> (sursa în `site/`) se reconstruiește
singură de workflow-ul **Pages** după fiecare rulare reușită a lui Release:
versiunea, linkurile fișierelor și noutățile vin din ultimul release publicat și
din `CHANGELOG*.md`. Dacă release-ului îi lipsește un fișier, pagina nu se
publică și rămâne cea veche. Pornire manuală: *Actions → Pages → Run workflow*.
Local:

```sh
gh release view --json tagName,publishedAt,url,assets > /tmp/release.json
python3 site/build.py --release /tmp/release.json --out _site
python3 -m http.server -d _site
```

## Ce se construiește

| Fișier | Pentru | Unde se construiește |
|---|---|---|
| `cangurul-<v>-linux-x86_64.AppImage` | Linux pe 64 de biți, un singur fișier executabil | `packaging/linux/appimage.sh`, ubuntu-22.04 |
| `cangurul-<v>-linux-x86_64.deb` | Ubuntu / Debian / Mint (Ubuntu 22.04+), cu meniu și asocierea fișierelor `.cng` | `packaging/linux/deb.sh`, ubuntu-22.04 |
| `cangurul-<v>-linux-x86_64.tar.gz` | Linux pe 64 de biți, arhivă simplă | `packaging/linux/package.sh`, ubuntu-22.04 |
| `cangurul-<v>-macos-universal.dmg` | macOS 10.13+, Apple Silicon și Intel (`Cangurul.app`) | `packaging/macos/package.sh`, macos-latest |
| `cangurul-<v>-windows-x64.msi` | Windows 10/11 pe 64 de biți, instalator: dosarul ales la instalare, meniul Start, scurtătura de pe desktop (opțională), asocierea fișierelor `.cng`, actualizare peste versiunea veche | `packaging/windows/msi.ps1 -Arch x64` (WiX Toolset v5) |
| `cangurul-<v>-windows-x86.msi` | Windows 10/11 pe 32 de biți, instalator | `packaging/windows/msi.ps1 -Arch x86` |
| `cangurul-<v>-windows-x64-portable.zip` | Windows 10/11 pe 64 de biți | `packaging/windows/package.ps1 -Arch x64` |
| `cangurul-<v>-windows-x86-portable.zip` | Windows 10/11 pe 32 de biți | `packaging/windows/package.ps1 -Arch x86` |
| `cangurul-<v>-windows7-x86-portable.zip` | Windows 7, 8, 10, 11 (32 și 64 de biți), cu Mesa pentru calculatoarele fără driver video | `packaging/windows7/package.sh`, container Arch Linux |
| `SHA256SUMS.txt` | sumele de control ale fișierelor de mai sus | jobul `release` |

Toate scripturile din `packaging/` pot fi rulate și local (cel pentru macOS
doar pe un Mac). Rezultatul ajunge în
`dist/release/`.

## De știut

- **Fișierele nu sunt semnate digital.** La prima pornire (și la instalarea
  `.msi`-ului), Windows SmartScreen va afișa „Windows a protejat computerul”; se continuă cu *Mai multe
  informații → Executare oricum*. Pe macOS, aplicația are doar o semnătură
  ad-hoc, deci Gatekeeper o blochează la prima pornire; ocolirea e descrisă în
  README (secțiunea macOS) și pe pagina programului. Semnarea și notarizarea
  reală cer un cont Apple Developer.
- `.msi`-ul primește doar partea numerică a versiunii (`3.1.0-rc.1` → `3.1.0`),
  pentru că MSI nu acceptă altceva. `UpgradeCode` din
  `packaging/windows/cangurul.wxs` nu se schimbă niciodată: după el, un `.msi` nou
  înlocuiește versiunea instalată.
- `appimagetool` se descarcă la prima rulare în `.cache/` (versiunea
  *continuous*). AppImage-ul conține doar programul; bibliotecile grafice
  (OpenGL, X11/Wayland) sunt luate din sistem. Fiind construit pe Ubuntu
  22.04, merge pe distribuțiile cu glibc 2.35 sau mai nou.
- Build-ul pentru Windows 7 folosește un Rust nightly fixat
  (`NIGHTLY` în `packaging/windows7/package.sh`), pentru că ținta
  `i686-win7-windows-gnu` nu e garantată stabilă. Nightly-ul se schimbă doar
  după un build de test reușit.
- Acțiunile GitHub sunt fixate pe SHA. Se actualizează manual, împreună cu
  comentariul cu versiunea de lângă SHA.
