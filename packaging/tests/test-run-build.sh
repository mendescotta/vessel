#!/bin/sh
set -eu

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
launcher="$script_dir/vessel-run-build"

out="$tmp/out"
mkdir -p "$out"
printf '#!/bin/sh\necho "ran in $(pwd)"\n' > "$out/build.sh"
chmod +x "$out/build.sh"
touch "$out/profile.toml"

output=$("$launcher" "$out")
case "$output" in
    "ran in $out") echo "PASS: runs build.sh from its folder" ;;
    *) echo "FAIL: unexpected output: $output"; exit 1 ;;
esac

rm "$out/profile.toml"
if "$launcher" "$out" 2>/dev/null; then
    echo "FAIL: should reject a folder without profile.toml"; exit 1
else
    echo "PASS: rejects folder without profile.toml"
fi

touch "$out/profile.toml"
chmod -x "$out/build.sh"
if "$launcher" "$out" 2>/dev/null; then
    echo "FAIL: should reject non-executable build.sh"; exit 1
else
    echo "PASS: rejects non-executable build.sh"
fi

if "$launcher" "$tmp/missing" 2>/dev/null; then
    echo "FAIL: should reject missing folder"; exit 1
else
    echo "PASS: rejects missing folder"
fi

if "$launcher" 2>/dev/null; then
    echo "FAIL: should reject no argument"; exit 1
else
    echo "PASS: rejects no argument"
fi

echo "All tests passed"
