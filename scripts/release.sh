#!/usr/bin/env sh
set -eu

usage() {
    printf '%s\n' 'Usage: scripts/release.sh [--push]' >&2
}

push=false
case "$#" in
    0) ;;
    1)
        if [ "$1" = '--push' ]; then
            push=true
        else
            usage
            exit 2
        fi
        ;;
    *)
        usage
        exit 2
        ;;
esac

[ "$(git branch --show-current)" = 'main' ] || {
    printf '%s\n' 'release preparation must run on main' >&2
    exit 1
}
[ -z "$(git status --porcelain)" ] || {
    printf '%s\n' 'release preparation requires a clean worktree' >&2
    exit 1
}

version=$(awk '
    $0 == "[workspace.package]" { inside = 1; next }
    inside && /^\[/ { exit }
    inside && $1 == "version" {
        gsub(/"/, "", $3)
        print $3
        exit
    }
' Cargo.toml)
[ -n "$version" ] || {
    printf '%s\n' 'Cargo.toml workspace.package version is missing' >&2
    exit 1
}

tag="v$version"
if git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
    printf '%s\n' "tag already exists: $tag" >&2
    exit 1
fi

previous_tag=$(git tag --merged HEAD --list 'v*' --sort=-version:refname | head -n 1 || true)
range=HEAD
[ -z "$previous_tag" ] || range="$previous_tag..HEAD"
changes=$(git log --no-merges --format='- %s (%h)' "$range")
[ -n "$changes" ] || changes='- No non-merge commits recorded.'

temporary=$(mktemp "${TMPDIR:-/tmp}/gamemanager-changelog.XXXXXX")
cleanup() {
    [ -z "${temporary:-}" ] || rm -f "$temporary"
}
trap cleanup 0 1 2 3 15

{
    printf '# Changelog\n\n'
    printf '## [%s] - %s\n\n' "$version" "$(date -u +%F)"
    printf '%s\n\n' "$changes"
    if [ -f CHANGELOG.md ]; then
        sed '1{/^# Changelog$/d;}' CHANGELOG.md
    fi
} >"$temporary"
mv "$temporary" CHANGELOG.md
temporary=

git add CHANGELOG.md
git commit -m "docs: update changelog for $tag"
git tag -a "$tag" -m "Release $tag"

if [ "$push" = true ]; then
    git push origin main --follow-tags
else
    printf '%s\n' "Created $tag locally. Push with: git push origin main --follow-tags"
fi
