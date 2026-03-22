#!/bin/sh
# devin-mcp installer
#
# Usage:
#   curl --proto '=https' --tlsv1.2 -LsSf \
#     https://github.com/mjinno09/devin-mcp/releases/latest/download/devin-mcp-installer.sh | sh
#
# The REPO and TAG placeholders are replaced at release time by the CD workflow.

set -eu

REPO="__REPO__"
TAG="__TAG__"

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "${os}" in
    Linux)  os_part="unknown-linux-gnu" ;;
    Darwin) os_part="apple-darwin" ;;
    *)      echo "Unsupported OS: ${os}" >&2; exit 1 ;;
  esac

  case "${arch}" in
    x86_64|amd64)   arch_part="x86_64" ;;
    aarch64|arm64)  arch_part="aarch64" ;;
    *)              echo "Unsupported architecture: ${arch}" >&2; exit 1 ;;
  esac

  echo "${arch_part}-${os_part}"
}

target="$(detect_target)"
archive="devin-mcp-${TAG}-${target}.tar.gz"
url="https://github.com/${REPO}/releases/download/${TAG}/${archive}"

echo "Downloading devin-mcp ${TAG} for ${target}..."

tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

curl --proto '=https' --tlsv1.2 -sSfL "${url}" -o "${tmpdir}/${archive}"
tar xzf "${tmpdir}/${archive}" -C "${tmpdir}"

install_dir="${HOME}/.cargo/bin"
mkdir -p "${install_dir}"
# Use cat+redirect instead of cp to avoid inheriting macOS quarantine
# extended attributes (com.apple.provenance) that can block execution.
cat "${tmpdir}/devin-mcp-${TAG}-${target}/devin-mcp" > "${install_dir}/devin-mcp"
chmod +x "${install_dir}/devin-mcp"

echo ""
echo "devin-mcp ${TAG} installed to ${install_dir}/devin-mcp"
echo ""
if echo "${PATH}" | tr ':' '\n' | grep -qx "${install_dir}"; then
  echo "Run 'devin-mcp --help' to get started."
else
  echo "Add ${install_dir} to your PATH, then run 'devin-mcp --help'."
fi
