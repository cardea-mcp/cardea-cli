#!/usr/bin/env bash

set -e

RED=$'\e[0;31m'
GREEN=$'\e[0;32m'
YELLOW=$'\e[0;33m'
NC=$'\e[0m' # No Color

info() {
	command printf '\e[0;32mInfo\e[0m: %s\n\n' "$1"
}

warn() {
	command printf '\e[0;33mWarn\e[0m: %s\n\n' "$1"
}

error() {
	command printf '\e[0;31mError\e[0m: %s\n\n' "$1" 1>&2
}

eprintf() {
	command printf '%s\n' "$1" 1>&2
}

_downloader() {
	local url=$1
	if ! command -v curl &>/dev/null; then
		if ! command -v wget &>/dev/null; then
			error "Cannot find wget or curl"
			eprintf "Please install wget or curl"
			exit 1
		else
			wget "$url"
		fi
	else
		curl --progress-bar -L -OC0 "$url"
	fi
}

get_latest_release() {
	echo "v0.1.1"
}

VERSION=$(get_latest_release)

check_os_arch() {
	[ -z "${ARCH}" ] && ARCH=$(uname -m)
	[ -z "${OS}" ] && OS=$(uname)
	RELEASE_FILE="x86_64-unknown-linux-gnu"

	case ${OS} in
		'Linux')
			case ${ARCH} in
				'x86_64') ARCH="x86_64";;
				'arm64' | 'armv8*' | 'aarch64') ARCH="aarch64" ;;
				'amd64') ARCH="x86_64" ;;
				*)
					error "Detected ${OS}-${ARCH} - currently unsupported"
					eprintf "Use --os and --arch to specify the OS and ARCH"
					exit 1
					;;
			esac
			RELEASE_FILE="${ARCH}-unknown-linux-gnu"

			;;
		'Darwin')
			case ${ARCH} in
				'x86_64') ARCH="x86_64" ;;
				'arm64' | 'arm' | 'aarch64') ARCH="aarch64" ;;
				*)
					error "Detected ${OS}-${ARCH} - currently unsupported"
					eprintf "Use --os and --arch to specify the OS and ARCH"
					exit 1
					;;
			esac
			RELEASE_FILE="${ARCH}-apple-darwin"

			;;
		'Windows_NT' | MINGW*)
			error "Detected ${OS} - currently unsupported"
			eprintf "Please download OpenMCP manually from the release page:"
			eprintf "https://github.com/decentralized-mcp/proxy/releases/latest"
			exit 1
			;;
		*)
			error "Detected ${OS}-${ARCH} - currently unsupported"
			eprintf "Use --os and --arch to specify the OS and ARCH"
			exit 1
			;;
	esac

	info "Detected ${OS}-${ARCH}"
	RELEASE_PKG="${RELEASE_FILE}.tgz"
}

main() {
	info "Fetching OpenMCP-$VERSION"
	check_os_arch
	_downloader "https://github.com/decentralized-mcp/proxy/releases/download/$VERSION/openmcp-$RELEASE_PKG"
	tar zxvf "openmcp-$RELEASE_PKG"
	if [ -d "$HOME/bin" ] && [ -w "$HOME/bin" ]; then
		mv "openmcp-$RELEASE_FILE/openmcp" "$HOME/bin/"
	else
		mv "openmcp-$RELEASE_FILE/openmcp" /usr/local/bin/
	fi
	rm -rf "openmcp-$RELEASE_FILE"
	rm -rf "openmcp-$RELEASE_PKG"
}


main "$@"

