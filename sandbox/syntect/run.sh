#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

resolve_abs() {
	local rel="$1"
	(cd "$rel" 2>/dev/null && pwd) || echo "$rel"
}

# Inject stylesheet link into generated HTML files (post-generation)
if [ -d static ]; then
	for f in out/*.html; do
		[ -f "$f" ] || continue
		if ! grep -q 'rel="stylesheet"' "$f"; then
			# Prepend link tag at the top of the file
			# Note: using sed -i to insert on the first line keeps it lightweight
			sed -i '1i <link rel="stylesheet" href="./style.css" />' "$f"
		fi
	done
fi

# Produce raw classed outputs into ./temp using env toggle
mkdir -p temp
for md in src/*.md; do
	[ -f "$md" ] || continue
	name=$(basename "$md" .md)
	PENDON_SYNTECT_DEBUG=classes cargo run --manifest-path ../../apps/cli/Cargo.toml -- \
		--plugin markdown,syntect --format html --pretty --input "$md" \
		> "temp/${name}.html"
done

# Inject stylesheet link into temp/*.html
for f in temp/*.html; do
	[ -f "$f" ] || continue
	if ! grep -q 'rel="stylesheet"' "$f"; then
		sed -i '1i <link rel="stylesheet" href="./style.css" />' "$f"
	fi
done
