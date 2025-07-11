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
			eprintf "Please download Cardea manually from the release page:"
			eprintf "https://github.com/cardea-mcp/cardea-cli/releases/latest/"
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
	info "Fetching Cardea"
	check_os_arch
	_downloader "https://github.com/cardea-mcp/cardea-cli/releases/latest/download/cardea-$RELEASE_PKG"
	tar zxvf "cardea-$RELEASE_PKG"
	if [ -d "$HOME/bin" ] && [ -w "$HOME/bin" ]; then
		mv "cardea-$RELEASE_FILE/cardea" "$HOME/bin/"
	else
		mv "cardea-$RELEASE_FILE/cardea" /usr/local/bin/
	fi
	rm -rf "cardea-$RELEASE_FILE"
	rm -rf "cardea-$RELEASE_PKG"
}


main "$@"

