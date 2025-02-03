#!/usr/bin/env bash
# Versiunea programului (singurul loc: [package] version din Cargo.toml).
#   packaging/version.sh              afișează versiunea
#   packaging/version.sh set 3.1.0    o schimbă în Cargo.toml și Cargo.lock
#   packaging/version.sh set 3.1.0-rc.1
set -euo pipefail
. "$(dirname "$0")/env.sh"
cd "$ROOT"

case "${1:-}" in
    "")
        echo "$VERSION"
        ;;
    set)
        v="${2:?Lipsește versiunea, de ex. 3.1.0}"
        v="${v#v}"
        if ! printf '%s' "$v" | grep -Eq '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$'; then
            echo "„$v” nu este o versiune de forma 1.2.3 sau 1.2.3-rc.1" >&2
            exit 1
        fi
        sed -i.bak "/^\[package\]/,/^\[/s/^version = \".*\"/version = \"$v\"/" Cargo.toml && rm Cargo.toml.bak
        cargo update --workspace --quiet
        echo "$VERSION -> $v"
        ;;
    *)
        echo "Folosire: $0 [set <versiune>]" >&2
        exit 2
        ;;
esac
