#!/bin/sh
# Update the project version everywhere: Cargo.toml, Cargo.lock,
# debian/changelog and packaging/patchsplit.spec.
# Usage: scripts/update-version.sh [X.Y.Z]  (prompts for a version if omitted)
set -eu

cd "$(dirname "$0")/.."

if [ $# -ge 1 ]; then
    version="$1"
else
    printf 'Enter new version (e.g. 1.3.0): '
    read -r version
fi

printf '%s' "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$' || {
    printf 'Error: invalid version: %s (expected X.Y.Z)\n' "$version" >&2
    exit 1
}

deb_suffix="1~ubuntu26.04.1"
deb_dist="resolute"
deb_maintainer="Oliver <oliver@liuxiaozhen.dev>"
rpm_maintainer="Oliver Lin <oliver@liuxiaozhen.dev>"

deb_date="$(LC_ALL=C date -R)"
rpm_date="$(LC_ALL=C date '+%a %b %d %Y')"

# Cargo.toml
sed -i "s/^version = .*/version = \"$version\"/" Cargo.toml

# Cargo.lock (version line right after the patchsplit package entry)
sed -i "/^name = \"patchsplit\"\$/{n;s/^version = .*/version = \"$version\"/;}" Cargo.lock

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

# debian/changelog: new entry on top, content left empty
{
    printf 'patchsplit (%s-%s) %s; urgency=medium\n\n  *\n\n -- %s  %s\n\n' \
        "$version" "$deb_suffix" "$deb_dist" "$deb_maintainer" "$deb_date"
    cat debian/changelog
} > "$tmp"
mv "$tmp" debian/changelog

# packaging/patchsplit.spec: Version field + new %changelog entry (content left empty)
sed -i "s/^Version:.*/Version:        $version/" packaging/patchsplit.spec
awk -v v="$version" -v d="$rpm_date" -v m="$rpm_maintainer" '
    /^%changelog$/ && !done {
        print
        printf "* %s %s - %s-1\n", d, m, v
        print ""
        done = 1
        next
    }
    { print }
' packaging/patchsplit.spec > "$tmp"
mv "$tmp" packaging/patchsplit.spec

printf 'Updated to %s:\n  Cargo.toml\n  Cargo.lock\n  debian/changelog\n  packaging/patchsplit.spec\n' "$version"
