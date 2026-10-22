#!/usr/bin/env bash
# Notele unei versiuni din CHANGELOG.md (pentru release-ul de pe GitHub).
#   packaging/changelog.sh            versiunea din Cargo.toml
#   packaging/changelog.sh 3.1.0
# Iese cu eroare dacă versiunea nu are secțiune în CHANGELOG.md.
set -euo pipefail
. "$(dirname "$0")/env.sh"
cd "$ROOT"

v="${1:-$VERSION}"
v="${v#v}"
notes="$(awk -v head="## [$v]" '
    index($0, head) == 1 { found = 1; next }
    found && /^## / { exit }
    found && /^\[[^]]+\]: / { exit }
    found { print }
' CHANGELOG.md | sed -e '/./,$!d' -e :a -e '/^\n*$/{$d;N;ba' -e '}')"

if [ -z "$notes" ]; then
    echo "CHANGELOG.md nu are secțiunea „## [$v]”." >&2
    exit 1
fi
printf '%s\n\n' "$notes"
echo "[English](https://github.com/nazdravan/cangurul/blob/main/CHANGELOG.en.md) · [Русский](https://github.com/nazdravan/cangurul/blob/main/CHANGELOG.ru.md)"
