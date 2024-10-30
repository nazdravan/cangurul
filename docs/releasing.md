# Cum se face un release

Release-urile se construiesc pe GitHub Actions (`.github/workflows/release.yml`),
după aceeași regulă ca la storytold/soundcraft:

- un push pe ramura `release` construiește arhivele pentru toate platformele și
  creează un **draft** GitHub Release „Cangurul v<versiune>”, cu tag-ul
  `v<versiune>`, fișierul `SHA256SUMS.txt` și note generate din commit-uri;
- draft-ul **nu se publică singur**: îl verifici și apeși *Publish release*;
- un release deja publicat nu poate fi suprascris. Dacă versiunea din
  `Cargo.toml` e deja publicată, workflow-ul se oprește din primul pas;
- o versiune cu sufix (`3.1.0-rc.1`) este marcată ca *pre-release*.

## Versiunea

Versiunea există într-un singur loc: `version` din secțiunea `[package]` a
fișierului `Cargo.toml`.

```sh
packaging/version.sh              # afișează versiunea
packaging/version.sh set 3.1.0    # o schimbă în Cargo.toml și Cargo.lock
```

## Pași

1. Pe `main`, mărește versiunea și fă commit:
   ```sh
   packaging/version.sh set 3.1.0
   git commit -am "Versiunea 3.1.0"
   git push
   ```
2. Adu ramura `release` la același commit:
   ```sh
   git push origin main:release
   ```
3. În tab-ul *Actions*, așteaptă să se termine workflow-ul **Release**
   (cel mai lung e build-ul pentru Windows 7, ~20–30 de minute).
4. În *Releases* apare draft-ul. Descarcă arhivele, încearcă-le, apoi apasă
   *Publish release*.

Dacă trebuie corectat ceva înainte de publicare, fă commit-ul, apoi din nou
`git push origin main:release`. Draft-ul existent se actualizează: i se
înlocuiesc fișierele.

## Rulare de test

Din *Actions → Release → Run workflow* se poate porni workflow-ul pe orice
ramură, opțional cu o versiune de test (de ex. `3.1.0-rc.1`). Pe altă ramură
decât `release` se fac doar testele și build-urile, iar arhivele rămân ca
artefacte ale rulării. Nu se creează niciun release.

## Ce se construiește

| Fișier | Pentru | Unde se construiește |
|---|---|---|
| `cangurul-<v>-linux-x86_64.tar.gz` | Linux pe 64 de biți | `packaging/linux/package.sh`, ubuntu-22.04 |
| `cangurul-<v>-windows-x64-portable.zip` | Windows 10/11 pe 64 de biți | `packaging/windows/package.ps1 -Arch x64` |
| `cangurul-<v>-windows-x86-portable.zip` | Windows 10/11 pe 32 de biți | `packaging/windows/package.ps1 -Arch x86` |
| `cangurul-<v>-windows7-x86-portable.zip` | Windows 7, 8, 10, 11 (32 și 64 de biți), cu Mesa pentru calculatoarele fără driver video | `packaging/windows7/package.sh`, container Arch Linux |
| `SHA256SUMS.txt` | sumele de control ale fișierelor de mai sus | jobul `release` |

Toate scripturile din `packaging/` pot fi rulate și local. Rezultatul ajunge în
`dist/release/`.

## De știut

- **Fișierele nu sunt semnate digital.** La prima pornire, Windows SmartScreen
  va afișa „Windows a protejat computerul”; se continuă cu *Mai multe
  informații → Executare oricum*.
- Build-ul pentru Windows 7 folosește un Rust nightly fixat
  (`NIGHTLY` în `packaging/windows7/package.sh`), pentru că ținta
  `i686-win7-windows-gnu` nu e garantată stabilă. Nightly-ul se schimbă doar
  după un build de test reușit.
- Acțiunile GitHub sunt fixate pe SHA. Se actualizează manual, împreună cu
  comentariul cu versiunea de lângă SHA.
