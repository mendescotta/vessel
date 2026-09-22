#!/bin/sh
set -eu

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

checkout="$tmp/checkout"
mkdir -p "$checkout"
cat > "$checkout/mkiso.sh" <<'EOF'
#!/bin/sh
echo "called with: $*"
EOF
chmod +x "$checkout/mkiso.sh"

argfile="$tmp/argv.txt"
printf '%s\n' "-a" "x86_64" "-b" "xfce" > "$argfile"

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
launcher="$script_dir/vessel-run-mkiso"

output=$("$launcher" "$argfile" "$checkout")
case "$output" in
    *"called with: -a x86_64 -b xfce"*) echo "PASS: forwards argv" ;;
    *) echo "FAIL: unexpected output: $output"; exit 1 ;;
esac

if "$launcher" "$tmp/missing-argfile" "$checkout" 2>/dev/null; then
    echo "FAIL: should reject missing argfile"; exit 1
else
    echo "PASS: rejects missing argfile"
fi

mkdir -p "$tmp/no-mkiso"
if "$launcher" "$argfile" "$tmp/no-mkiso" 2>/dev/null; then
    echo "FAIL: should reject checkout without mkiso.sh"; exit 1
else
    echo "PASS: rejects checkout without mkiso.sh"
fi

echo "All tests passed"
