#!/bin/sh
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
root="${DESTDIR:-}"
launcher="$root/usr/libexec/vessel/run-build"
policy="$root/usr/share/polkit-1/actions/org.voidlinux.vessel.policy"

case "${1:-}" in
"")
	install -Dm755 "$here/vessel-run-build" "$launcher"
	install -Dm644 "$here/org.voidlinux.vessel.policy" "$policy"
	echo "Installed $launcher"
	echo "Installed $policy"
	;;
--uninstall)
	rm -f "$launcher" "$policy"
	rmdir "$root/usr/libexec/vessel" 2>/dev/null || true
	echo "Removed $launcher and $policy"
	;;
*)
	echo "usage: [sudo] $0 [--uninstall]" >&2
	exit 2
	;;
esac
