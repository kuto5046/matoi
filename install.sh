#!/bin/sh

set -eu

REPO="${MATOI_REPO:-kuto5046/matoi}"
BIN_NAME="matoi"
INSTALL_DIR="${MATOI_INSTALL_DIR:-$HOME/.local/bin}"
BASE_URL="https://github.com/${REPO}/releases/latest/download"

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "error: '$1' is required" >&2
    exit 1
  fi
}

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)
      os_part="unknown-linux-gnu"
      ;;
    Darwin)
      os_part="apple-darwin"
      ;;
    *)
      echo "error: unsupported OS: $os" >&2
      exit 1
      ;;
  esac

  case "$arch" in
    x86_64|amd64)
      arch_part="x86_64"
      ;;
    arm64|aarch64)
      arch_part="aarch64"
      ;;
    *)
      echo "error: unsupported architecture: $arch" >&2
      exit 1
      ;;
  esac

  printf '%s-%s\n' "$arch_part" "$os_part"
}

sha256_file() {
  file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | awk '{print $1}'
  else
    shasum -a 256 "$file" | awk '{print $1}'
  fi
}

path_contains() {
  dir="$1"
  old_ifs=${IFS}
  IFS=:
  for p in $PATH; do
    if [ "$p" = "$dir" ]; then
      IFS=${old_ifs}
      return 0
    fi
  done
  IFS=${old_ifs}
  return 1
}

need_cmd curl
need_cmd tar
need_cmd mktemp
need_cmd chmod
need_cmd cp
need_cmd awk
need_cmd uname

if ! command -v sha256sum >/dev/null 2>&1 && ! command -v shasum >/dev/null 2>&1; then
  echo "error: sha256sum or shasum is required" >&2
  exit 1
fi

target="$(detect_target)"
asset="${BIN_NAME}-${target}.tar.gz"
checksum_asset="${asset}.sha256"

tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/matoi-install.XXXXXX")"
cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT INT TERM

echo "Downloading ${asset}..."
curl -fsSL "${BASE_URL}/${asset}" -o "${tmpdir}/${asset}"
curl -fsSL "${BASE_URL}/${checksum_asset}" -o "${tmpdir}/${checksum_asset}"

expected="$(awk '{print $1}' "${tmpdir}/${checksum_asset}")"
actual="$(sha256_file "${tmpdir}/${asset}")"
if [ "$expected" != "$actual" ]; then
  echo "error: checksum verification failed" >&2
  exit 1
fi

mkdir -p "$INSTALL_DIR"
tar -xzf "${tmpdir}/${asset}" -C "$tmpdir"
chmod +x "${tmpdir}/${BIN_NAME}"
cp "${tmpdir}/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"

echo "Installed ${BIN_NAME} to ${INSTALL_DIR}/${BIN_NAME}"
if ! path_contains "$INSTALL_DIR"; then
  echo "Note: ${INSTALL_DIR} is not in PATH"
fi
