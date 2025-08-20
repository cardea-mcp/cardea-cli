#!/usr/bin/env bash

set -Eeuo pipefail
IFS=$'\n\t'

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
	# Use default-empty expansion to avoid set -u errors when ARCH/OS are unset
	[ -z "${ARCH:-}" ] && ARCH=$(uname -m)
	[ -z "${OS:-}" ] && OS=$(uname)
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

update_shell_configs() {
	local install_path=$1
	local path_export="export PATH=\"$install_path:\$PATH\""
	local path_check="# Check if $install_path is in PATH
	if [[ \":\$PATH:\" != *\":$install_path:\"* ]]; then
		$path_export
		fi"

		# Track if we updated any config
		local updated=0

		# Function to update a config file
		update_config_file() {
			local config_file=$1
			local config_path="$HOME/$config_file"

			if [ -f "$config_path" ] || [ "$config_file" = ".bash_profile" ] || [ "$config_file" = ".profile" ]; then
				if ! grep -Fq "$install_path" "$config_path" 2>/dev/null; then
					[ ! -f "$config_path" ] && touch "$config_path"
					info "Adding $install_path to PATH in $config_file"
					echo "" >> "$config_path"
					echo "# Added by Cardea installer" >> "$config_path"
					echo "$path_check" >> "$config_path"
					updated=1
				else
					info "$install_path already in $config_file"
				fi
			fi
		}

		# Detect shell and update appropriate config files
		local current_shell="${SHELL##*/}"

		case "$current_shell" in
			zsh)
				# Prefer .zprofile for login shells, .zshrc for interactive
				update_config_file ".zshrc"
				update_config_file ".zprofile"
				;;
			bash)
				# For bash, update .bashrc and .bash_profile
				update_config_file ".bashrc"
				update_config_file ".bash_profile"
				;;
			*)
				# For other shells, update .profile
				update_config_file ".profile"
				;;
		esac

		return $updated
	}

main() {
	info "Fetching Cardea"
	check_os_arch

	# Ensure required tools exist
	if ! command -v tar >/dev/null 2>&1; then
		error "tar not found"
		eprintf "Please install tar and re-run the installer."
		exit 1
	fi

	_downloader "https://github.com/cardea-mcp/cardea-cli/releases/latest/download/cardea-$RELEASE_PKG"
	tar zxvf "cardea-$RELEASE_PKG"

	# Choose install destination
	local dest=""
	case ${OS} in
		Linux)
			dest="${XDG_BIN_HOME:-$HOME/.local/bin}"
			;;
		Darwin)
			dest="$HOME/bin"
			;;
		*)
			dest="$HOME/bin"
			;;
	esac

	mkdir -p "$dest" || true

	do_install() {
		if command -v install >/dev/null 2>&1; then
			install -m 0755 "cardea-$RELEASE_FILE/cardea" "$1/"
		else
			cp "cardea-$RELEASE_FILE/cardea" "$1/"
		fi
	}

	local installed_to=""
	if [ -w "$dest" ]; then
		do_install "$dest"
		installed_to="$dest"
		info "Cardea installed to $dest/cardea"
		update_shell_configs "$dest"
	elif [ -w "/usr/local/bin" ]; then
		do_install "/usr/local/bin"
		installed_to="/usr/local/bin"
		info "Cardea installed to /usr/local/bin/cardea"
	else
		error "No writable install location found"
		eprintf "Tried: $dest and /usr/local/bin"
		eprintf "Re-run with appropriate permissions or choose a writable DEST in your PATH."
		exit 1
	fi

	# Clean up temporary files
	rm -rf "cardea-$RELEASE_FILE"
	rm -rf "cardea-$RELEASE_PKG"

	echo ""
	echo "${GREEN}Installation successful!${NC}"

	if [[ ":$PATH:" != *":$installed_to:"* ]]; then
		echo ""
		echo "${YELLOW}Note:${NC} $installed_to has been added (or ensured) in your shell configuration."
		echo "To use cardea immediately, run one of the following:"
		echo "  ${GREEN}source ~/.bashrc${NC}  (bash)"
		echo "  ${GREEN}source ~/.zshrc${NC}   (zsh)"
		echo "  Or start a new terminal session"
	else
		echo "Cardea is ready to use!"
	fi

	echo ""
	echo "Run ${GREEN}cardea --help${NC} to get started"
}

main "$@"
