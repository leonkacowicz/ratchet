#!/bin/sh
# ratchet installer — download a prebuilt release binary for this platform,
# verify its SHA-256 checksum, and drop `ratchet` on your PATH.
#
#   curl -fsSL https://raw.githubusercontent.com/leonkacowicz/ratchet/main/install.sh | sh
#
# Options (flags or the matching env vars):
#   --version <tag>    RATCHET_VERSION   release tag to install (default: latest)
#   --bin-dir <dir>    RATCHET_BIN_DIR   install dir (default: $PREFIX/bin or ~/.local/bin)
#   --dry-run                            print what would happen, then exit
#   -h, --help                           show this help
#
# Windows is not covered by this POSIX script — use the reusable action or grab the
# .zip from the releases page directly.

set -eu

REPO="${RATCHET_REPO:-leonkacowicz/ratchet}"
VERSION="${RATCHET_VERSION:-latest}"
# Default install dir: honour $PREFIX (→ $PREFIX/bin) if the caller set one, else
# the conventional per-user bin dir.
if [ -n "${RATCHET_BIN_DIR:-}" ]; then
	BIN_DIR="$RATCHET_BIN_DIR"
elif [ -n "${PREFIX:-}" ]; then
	BIN_DIR="$PREFIX/bin"
else
	BIN_DIR="$HOME/.local/bin"
fi
DRY_RUN=0

log() { printf 'ratchet-install: %s\n' "$1" >&2; }
die() {
	printf 'ratchet-install: error: %s\n' "$1" >&2
	exit 1
}
have() { command -v "$1" >/dev/null 2>&1; }

usage() {
	cat <<'EOF'
ratchet installer — download a prebuilt release binary for this platform,
verify its SHA-256 checksum, and drop `ratchet` on your PATH.

  curl -fsSL https://raw.githubusercontent.com/leonkacowicz/ratchet/main/install.sh | sh

Options (flags or the matching env vars):
  --version <tag>    RATCHET_VERSION   release tag to install (default: latest)
  --bin-dir <dir>    RATCHET_BIN_DIR   install dir (default: $PREFIX/bin or ~/.local/bin)
  --dry-run                            print what would happen, then exit
  -h, --help                           show this help

Windows is not covered by this POSIX script — use the reusable action or grab the
.zip from the releases page directly.
EOF
	exit "${1:-0}"
}

# --- parse arguments -------------------------------------------------------
while [ $# -gt 0 ]; do
	case "$1" in
	--version)
		VERSION="${2:-}"
		shift 2 || die "--version needs a value"
		;;
	--version=*) VERSION="${1#*=}" && shift ;;
	--bin-dir)
		BIN_DIR="${2:-}"
		shift 2 || die "--bin-dir needs a value"
		;;
	--bin-dir=*) BIN_DIR="${1#*=}" && shift ;;
	--dry-run) DRY_RUN=1 && shift ;;
	-h | --help) usage 0 ;;
	*) die "unknown option: $1 (try --help)" ;;
	esac
done

# --- detect platform → release target triple -------------------------------
os="$(uname -s)"
arch="$(uname -m)"
case "$os" in
Linux)
	case "$arch" in
	x86_64 | amd64) target="x86_64-unknown-linux-gnu" ;;
	*) die "no prebuilt Linux binary for '$arch'. Build from source: cargo install --git https://github.com/$REPO --locked" ;;
	esac
	;;
Darwin)
	case "$arch" in
	arm64 | aarch64) target="aarch64-apple-darwin" ;;
	x86_64) target="x86_64-apple-darwin" ;;
	*) die "no prebuilt macOS binary for '$arch'" ;;
	esac
	;;
*) die "unsupported OS '$os'. On Windows, download the .zip from https://github.com/$REPO/releases" ;;
esac

# --- pick a downloader -----------------------------------------------------
if have curl; then
	http_dl() { curl -fsSL -o "$2" "$1"; }
	# Final URL after following redirects — for …/releases/latest that is …/releases/tag/<tag>.
	final_url() { curl -fsSLI -o /dev/null -w '%{url_effective}' "$1"; }
elif have wget; then
	http_dl() { wget -qO "$2" "$1"; }
	# wget follows redirects with --spider; the last Location header is the tag URL.
	final_url() { wget -qS --spider "$1" 2>&1 | sed -n 's/^[[:space:]]*[Ll]ocation:[[:space:]]*//p' | tail -n1; }
else
	die "need curl or wget on PATH"
fi

# --- resolve the release tag ----------------------------------------------
if [ "$VERSION" = latest ]; then
	log "resolving latest release of $REPO ..."
	# Resolve via the github.com redirect rather than api.github.com, which has a
	# tight unauthenticated rate limit that 403s on shared CI runner IPs.
	VERSION="$(final_url "https://github.com/$REPO/releases/latest" | sed -n 's#.*/releases/tag/##p' | tr -d '[:space:]')"
	[ -n "$VERSION" ] || die "could not resolve the latest release tag (is a release published yet?)"
fi

asset="ratchet-$VERSION-$target.tar.gz"
base_url="https://github.com/$REPO/releases/download/$VERSION"

log "platform : $os/$arch → $target"
log "release  : $VERSION"
log "asset    : $asset"
log "install  : $BIN_DIR/ratchet"

if [ "$DRY_RUN" -eq 1 ]; then
	log "dry run — not downloading. URL: $base_url/$asset"
	exit 0
fi

# --- checksum tool ---------------------------------------------------------
if have sha256sum; then
	sha_check() { sha256sum -c "$1"; }
elif have shasum; then
	sha_check() { shasum -a 256 -c "$1"; }
else
	die "need sha256sum or shasum to verify the download"
fi

# --- download, verify, install --------------------------------------------
tmp="$(mktemp -d "${TMPDIR:-/tmp}/ratchet-install.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT INT TERM

log "downloading ..."
http_dl "$base_url/$asset" "$tmp/$asset" || die "download failed: $base_url/$asset"
http_dl "$base_url/$asset.sha256" "$tmp/$asset.sha256" || die "checksum download failed"

# The .sha256 names the archive by basename, so verify from inside the temp dir.
log "verifying checksum ..."
(cd "$tmp" && sha_check "$asset.sha256") || die "checksum verification FAILED — refusing to install"

log "extracting ..."
tar -xzf "$tmp/$asset" -C "$tmp" || die "extract failed"
[ -f "$tmp/ratchet" ] || die "archive did not contain a 'ratchet' binary"

mkdir -p "$BIN_DIR" || die "could not create $BIN_DIR"
install -m 0755 "$tmp/ratchet" "$BIN_DIR/ratchet" 2>/dev/null ||
	{ cp "$tmp/ratchet" "$BIN_DIR/ratchet" && chmod 0755 "$BIN_DIR/ratchet"; } ||
	die "could not install to $BIN_DIR (try --bin-dir or sudo)"

log "installed ratchet $VERSION to $BIN_DIR/ratchet"

# Nudge if the install dir isn't on PATH.
case ":$PATH:" in
*":$BIN_DIR:"*) : ;;
*) log "note: $BIN_DIR is not on your PATH — add it to use 'ratchet' directly" ;;
esac
