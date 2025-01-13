#!/usr/bin/env bash

###############################################################################
# COLORS
###############################################################################
GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[0;33m"
RESET="\033[0m"

###############################################################################
# INTERRUPT HANDLER
###############################################################################
int_handler() {
	echo -e "\n${YELLOW}Interrupted. Exiting.${RESET}"
	# Just exit rather than killing the parent process to be less destructive.
	exit 1
}
trap 'int_handler' INT

###############################################################################
# WELCOME
###############################################################################
echo -e "${GREEN}Welcome to hyveOS the installation script!${RESET}"

###############################################################################
# CONTINUE INSTALLATION FUNCTION
###############################################################################
function continue_installation() {
	# Default is yes; if the user presses enter, it continues.
	read -p "Continue (Y/n)? " -n 1 -r
	echo

	# If empty or Y/y, continue
	if [[ -z "$REPLY" || "$REPLY" =~ ^[Yy]$ ]]; then
		echo -e "${GREEN}Continuing installation${RESET}"
	else
		echo -e "${RED}Exiting installation${RESET}"
		exit 1
	fi
}

###############################################################################
# OS CHECK
###############################################################################
if [[ "$(uname)" == "Darwin" ]]; then
	echo -e "${RED}This script is not supported on MacOS${RESET}"
	exit 1
elif [[ "$(uname)" == "Linux" ]]; then
	echo -e "${GREEN}Linux detected${RESET}"
else
	echo -e "${RED}Unsupported OS${RESET}"
	echo -e "${RED}Proceed at your own risk?${RESET}"
	continue_installation
fi

###############################################################################
# DISTRO CHECK (Ubuntu, Debian, Raspberry Pi OS)
###############################################################################
if [ -f /etc/os-release ]; then
	. /etc/os-release
	OS=$NAME
	VER=$VERSION_ID

	case "$OS" in
	"Ubuntu")
		echo -e "${GREEN}Ubuntu detected${RESET}"
		if [[ "$VER" == "22.04" ]]; then
			echo -e "${GREEN}Ubuntu 22.04 detected${RESET}"
		elif [[ "$VER" == "24.04" ]]; then
			echo -e "${GREEN}Ubuntu 24.04 detected${RESET}"
		else
			echo -e "${RED}Unsupported Ubuntu version ($VER)${RESET}"
			echo -e "${RED}Proceed at your own risk?${RESET}"
			continue_installation
		fi
		;;
	"Debian GNU/Linux")
		echo -e "${GREEN}Debian detected (Version: $VER)${RESET}"
		# You could optionally check if $VER -ge 11 (Bullseye), etc.
		# For now we just proceed.
		;;
	"Raspbian GNU/Linux" | "Raspberry Pi OS")
		echo -e "${GREEN}Raspberry Pi OS / Raspbian detected${RESET}"
		;;
	*)
		echo -e "${RED}Unsupported distro: $OS${RESET}"
		echo -e "${RED}Proceed at your own risk?${RESET}"
		continue_installation
		;;
	esac
else
	echo -e "${RED}No /etc/os-release found, can't determine OS${RESET}"
	echo -e "${RED}Proceed at your own risk?${RESET}"
	continue_installation
fi

###############################################################################
# ARCH CHECK
###############################################################################
ARCH=$(uname -m)
if [[ "$ARCH" == "aarch64" ]]; then
	echo -e "${GREEN}ARM64 (aarch64) detected${RESET}"
elif [[ "$ARCH" == "x86_64" ]]; then
	echo -e "${GREEN}x86_64 detected${RESET}"
else
	echo -e "${RED}Unsupported architecture: $ARCH${RESET}"
	echo -e "${YELLOW}Supported architectures: aarch64, x86_64${RESET}"
	echo -e "${RED}Proceed at your own risk?${RESET}"
	continue_installation
fi

###############################################################################
# INSTALL DEPENDENCIES
###############################################################################
read -p "Install dependencies (wget, curl, gpg, jq)? (y/n) " -n 1 -r
echo
if [[ "$REPLY" =~ ^[Yy]$ ]]; then
	echo -e "${GREEN}Installing dependencies${RESET}"
	sudo apt update -y
	sudo apt install -y wget curl gpg jq
else
	echo -e "${RED}This script may not run properly without these dependencies.${RESET}"
	echo -e "${RED}Please install hyveOS manually instead.${RESET}"
	exit 1
fi

###############################################################################
# CHECK DOCKER
###############################################################################
if docker --version &>/dev/null; then
	echo -e "${GREEN}Docker is already installed${RESET}"
else
	echo -e "${RED}Docker is not installed${RESET}"
	echo -e "${YELLOW}Options:${RESET}"
	echo -e "1) Install Docker yourself (recommended)."
	echo -e "2) Install Docker using convenience script (easier, less secure)."
	echo -e "3) Exit installation."

	read -p "Choose an option (1/2/3): " -n 1 -r
	echo

	case "$REPLY" in
	"1")
		echo -e "${YELLOW}Please install Docker manually via:${RESET}"
		echo -e "  https://docs.docker.com/engine/install/"
		if [[ -x "$(command -v xdg-open)" ]]; then
			xdg-open https://docs.docker.com/engine/install/
		fi
		echo "After installing Docker, press ENTER to continue."
		read
		;;
	"2")
		echo -e "${YELLOW}Installing Docker using convenience script${RESET}"
		curl -sSL https://get.docker.com | sh ||
			(echo -e "${RED}Error installing Docker${RESET}" && exit 1)
		echo -e "${GREEN}Docker installed successfully${RESET}"
		;;
	*)
		echo -e "${RED}Exiting installation${RESET}"
		exit 1
		;;
	esac
fi

###############################################################################
# MESH SUPPORT (BATMAN-ADV)
###############################################################################
read -p "Do you want to install mesh support (batctl, batman-adv)? (y/n) " -n 1 -r
echo
MESH_SUPPORT=false
if [[ "$REPLY" =~ ^[Yy]$ ]]; then
	echo -e "${GREEN}Installing mesh support (batctl, batman-adv)${RESET}"

	# Install batctl if missing
	if ! command -v batctl &>/dev/null; then
		sudo apt update -y
		sudo apt install -y batctl
	else
		echo -e "${GREEN}batctl is already installed${RESET}"
	fi

	# Check if batman-adv kernel module is available
	if ! modinfo batman-adv &>/dev/null; then
		echo -e "${YELLOW}batman-adv kernel module not found.${RESET}"
		echo -e "${YELLOW}Often installed via 'linux-modules-extra-$(uname -r)'.${RESET}"
		read -p "Install it now? (y/n) " -n 1 -r
		echo
		if [[ "$REPLY" =~ ^[Yy]$ ]]; then
			sudo apt update -y
			sudo apt install -y "linux-modules-extra-$(uname -r)"
			if modinfo batman-adv &>/dev/null; then
				echo -e "${GREEN}batman-adv installed successfully${RESET}"
				MESH_SUPPORT=true
			else
				echo -e "${RED}Failed to install batman-adv${RESET}"
				echo -e "${RED}Proceeding without mesh support${RESET}"
			fi
		fi
	else
		echo -e "${GREEN}Batman-adv kernel module is already present${RESET}"
		MESH_SUPPORT=true
	fi

	# If batman-adv is available, ensure it's loaded
	if [[ "$MESH_SUPPORT" == true ]]; then
		if ! lsmod | grep -q batman_adv; then
			sudo modprobe batman-adv
			echo -e "${GREEN}Loaded batman-adv kernel module${RESET}"
			echo -e "${GREEN}Ensuring batman-adv loads at boot${RESET}"
			sudo sh -c "echo batman-adv >> /etc/modules"
		else
			echo -e "${GREEN}Batman-adv is already loaded${RESET}"
		fi
	fi
else
	echo -e "${RED}Skipping mesh support${RESET}"
fi

###############################################################################
# CHECK IF HYVED IS INSTALLED
###############################################################################
if command -v hyved &>/dev/null; then
	echo -e "${GREEN}Hyved is already installed${RESET}"
else
	echo -e "${RED}Hyved is not installed${RESET}"
	echo -e "${YELLOW}Installing Hyved now${RESET}"
	echo -e "${GREEN}Adding GPG key for p2p.industries${RESET}"

	echo "deb https://apt.p2p.industries /" | sudo tee /etc/apt/sources.list.d/p2p-industries.list
	wget -qO - https://apt.p2p.industries/key.gpg | sudo gpg --dearmor --yes -o /etc/apt/trusted.gpg.d/p2p-industries.gpg
	sudo apt update -y
	sudo apt install -y hyved hyvectl
fi

###############################################################################
# OPTIONAL: CONFIGURE WIFI INTERFACE
###############################################################################
echo -e "${RED}WARNING:${RESET} Configuring a Wi-Fi interface can break your connection if you're using it right now."
echo -e "Make sure you have another connection method or are OK with losing access."

read -p "Do you want to configure the Wi-Fi interface now? (y/n) " -n 1 -r
echo
if [[ "$REPLY" =~ ^[Yy]$ ]]; then
	echo -e "${GREEN}Configuring Wi-Fi interface${RESET}"
	# Example: 'hyvectl init --json' => {"wifi_interface":"wlan0",...}
	OUTPUT=$(sudo hyvectl --json init)
	wifi_interface=$(echo "$OUTPUT" | jq -r '.wifi_interface')

	echo "Detected Wi-Fi interface: $wifi_interface"
	if [ ! -d "/sys/class/net/$wifi_interface" ]; then
		echo -e "${RED}Interface $wifi_interface does not exist${RESET}"
		exit 1
	fi

	if [ ! -d "/sys/class/net/$wifi_interface/wireless/" ]; then
		echo -e "${RED}Interface $wifi_interface is not wireless${RESET}"
		exit 1
	fi

	if iw "$wifi_interface" info 2>/dev/null | grep -q "type managed"; then
		echo "Interface is in 'managed' mode Wi-Fi."
	else
		echo "Interface is Wi-Fi but not in 'managed' mode."
		echo "wpa_supplicant may not work until set to 'managed' mode."

		###############################################################################
		# SNIPPET: Prompt user to remove Wi-Fi interface from Netplan (if present)
		###############################################################################
		if [ -d /etc/netplan ] && ls /etc/netplan/*.yaml &>/dev/null; then
			echo -e "${YELLOW}It appears Netplan is in use and may be managing your Wi-Fi interface.${RESET}"
			echo -e "To allow wpa_supplicant (or systemd-networkd) to manage ${wifi_interface},"
			echo -e "you should remove or comment out any configuration for ${wifi_interface} in Netplan."
			echo
			for npfile in /etc/netplan/*.yaml; do
				echo -e "Reviewing Netplan file: ${GREEN}${npfile}${RESET}"
				echo "----- Current contents -----"
				cat "$npfile"
				echo "----------------------------"
				# If EDITOR is not set, use nano as the default
				EDITOR=${EDITOR:-nano}
				echo -e "${YELLOW}Press ENTER to open this file in ${EDITOR}. Remove references to ${wifi_interface}, then save & exit.${RESET}"
				read # Wait for user to press ENTER
				sudo $EDITOR "$npfile"
			done

			echo -e "${YELLOW}Applying Netplan changes now...${RESET}"
			# Note: This might break connectivity if you're currently on that interface.
			sudo netplan apply || true
			echo "Netplan changes applied."
		else
			echo -e "${GREEN}No Netplan YAML files found, or Netplan is not in use. Nothing to remove.${RESET}"
		fi
	fi

	# Copy default config for wpa_supplicant
	sudo cp /usr/lib/hyved/wpa_supplicant-generic.conf \
		/etc/wpa_supplicant/wpa_supplicant-$wifi_interface.conf

	# Enable wpa_supplicant for this interface
	sudo systemctl enable --now wpa_supplicant@$wifi_interface

	# If also using batman on Wi-Fi, enable the hyveos-batman service
	sudo systemctl enable --now hyveos-batman@"$wifi_interface"
fi

###############################################################################
# ENABLE DOCKER & BATMAN NEIGHBOURS
###############################################################################
echo -e "${GREEN}Enabling Docker & batman-neighbours-daemon at boot${RESET}"
continue_installation
sudo systemctl enable --now docker
sudo systemctl enable --now batman-neighbours-daemon

echo -e "${GREEN}Installation steps complete${RESET}"
echo "To start hyved now:   sudo systemctl start hyved"
echo "To enable hyved boot: sudo systemctl enable --now hyved"

###############################################################################
# PROMPT FOR BRIDGE SETUP (ETH + OPTIONAL WIFI)
###############################################################################
echo -e "${RED}WARNING:${RESET} If you create a bridge, any interface you add (Ethernet or Wi-Fi) will likely lose normal connectivity as it becomes enslaved to the bridge."
echo -e "Make sure you have an alternative means to access the machine."

read -p "Do you want to setup this node as a bridge? (y/n) " -n 1 -r
echo
if [[ "$REPLY" =~ ^[Yy]$ ]]; then
	set -e # exit on error

	###########################################################################
	# 1. Prompt user to remove Ethernet/Wi-Fi from netplan if on Ubuntu
	#    (Debian or Raspbian might not use netplan by default, but let's check)
	###########################################################################
	if [ -d /etc/netplan ] && ls /etc/netplan/*.yaml &>/dev/null; then
		echo -e "${YELLOW}Detected netplan files (commonly on Ubuntu).${RESET}"
		echo "You must remove or comment out the configuration for the interface(s)"
		echo "you want systemd-networkd to manage. Otherwise, netplan will conflict."

		for npfile in /etc/netplan/*.yaml; do
			EDITOR=${EDITOR:-nano}
			echo -e "${GREEN}Opening $npfile in $EDITOR...${RESET}"
			echo "----- Current $npfile contents -----"
			sudo cat "$npfile"
			echo "------------------------------------"
			echo "Press ENTER to edit $npfile"
			read
			sudo $EDITOR "$npfile"
		done

	fi

	# Optionally: disable NetworkManager if you want only systemd-networkd
	# sudo systemctl enable systemd-networkd
	# sudo systemctl disable NetworkManager

	###########################################################################
	# 3. Detect an Ethernet interface with Internet
	###########################################################################
	echo "Detecting an active Ethernet interface with Internet connectivity..."

	DEFAULT_IFACE=""
	# Grab default route
	CANDIDATE_IFACE=$(ip route show default 2>/dev/null | awk '/default/ {print $5; exit}')

	if [ -n "$CANDIDATE_IFACE" ]; then
		if ip link show "$CANDIDATE_IFACE" | grep -q "link/ether"; then
			echo "Candidate default route interface: $CANDIDATE_IFACE"
			echo "Testing Internet connectivity..."
			if curl --interface "$CANDIDATE_IFACE" -s --max-time 5 https://ifconfig.me >/dev/null; then
				DEFAULT_IFACE="$CANDIDATE_IFACE"
			else
				echo "No connectivity on $CANDIDATE_IFACE"
			fi
		fi
	fi

	# If no good candidate found, try scanning
	if [ -z "$DEFAULT_IFACE" ]; then
		for IFACE in $(ls /sys/class/net | grep -E '^eth|^en'); do
			if ip link show "$IFACE" | grep -q "state UP"; then
				if curl --interface "$IFACE" -s --max-time 5 http://ifconfig.me >/dev/null; then
					DEFAULT_IFACE="$IFACE"
					break
				fi
			fi
		done
	fi

	if [ -z "$DEFAULT_IFACE" ]; then
		echo -e "${RED}ERROR: No Ethernet interface with Internet found.${RESET}"
		echo "Please configure bridging manually."
		exit 1
	fi

	echo "Using '$DEFAULT_IFACE' as the main Ethernet interface with Internet."

	###########################################################################
	# 4. Optionally detect a Wi-Fi interface for bridging
	###########################################################################
	echo -e "${YELLOW}Do you also want to add a Wi-Fi interface to the bridge?${RESET}"
	echo -e "(Bridging a Wi-Fi interface in managed mode usually doesn't work as a real L2 bridge.)"
	read -p "Add Wi-Fi to the bridge as well? (y/n) " -n 1 -r
	echo

	WIFI_BRIDGE_IFACE=""
	if [[ "$REPLY" =~ ^[Yy]$ ]]; then
		read -p "Enter the Wi-Fi interface name (e.g., wlan0): " WIFI_BRIDGE_IFACE
		echo "Wi-Fi interface to be added: $WIFI_BRIDGE_IFACE"
		echo "If it’s in managed mode, bridging may degrade or fail."
		echo
	fi

	###########################################################################
	# 5. Create systemd-networkd configurations for bridging
	###########################################################################
	read -p "Enter the desired bridge name [default: br0]: " BRIDGE_NAME
	BRIDGE_NAME=${BRIDGE_NAME:-br0}
	echo "Using bridge name: $BRIDGE_NAME"

	sudo mkdir -p /etc/systemd/network

	# Bridge netdev
	cat <<EOF | sudo tee /etc/systemd/network/10-$BRIDGE_NAME.netdev
[NetDev]
Name=$BRIDGE_NAME
Kind=bridge
EOF

	# Bridge network (gets DHCP)
	cat <<EOF | sudo tee /etc/systemd/network/10-$BRIDGE_NAME.network
[Match]
Name=$BRIDGE_NAME

[Network]
DHCP=ipv4
EOF

	# Enslave the main Ethernet interface
	cat <<EOF | sudo tee /etc/systemd/network/20-$DEFAULT_IFACE.network
[Match]
Name=$DEFAULT_IFACE

[Network]
DHCP=no
Bridge=$BRIDGE_NAME
EOF

	# If bat0 exists, enslave it (no DHCP)
	if ip link show bat0 &>/dev/null; then
		cat <<EOF | sudo tee /etc/systemd/network/30-bat0.network
[Match]
Name=bat0

[Network]
DHCP=no
Bridge=$BRIDGE_NAME
EOF
	fi

	# If user wants to add Wi-Fi
	if [[ -n "$WIFI_BRIDGE_IFACE" ]]; then
		cat <<EOF | sudo tee /etc/systemd/network/40-$WIFI_BRIDGE_IFACE.network
[Match]
Name=$WIFI_BRIDGE_IFACE

[Network]
DHCP=no
Bridge=$BRIDGE_NAME
EOF
		echo -e "${YELLOW}WARNING: Bridging a Wi-Fi client interface is not a true L2 bridge in most drivers.${RESET}"
	fi

	#### Ask the user to restart after this step ####
	echo "For the changes to take effect, you must restart. Since this might break connectivity."
	echo -e "${YELLOW}Please ensure you have another means to access the machine.${RESET}"
	continue_installation

	###########################################################################
	# 6. Enable and restart systemd-networkd
	###########################################################################
	echo "Applying netplan changes (this might break connectivity). Wait a few seconds..."
	sudo netplan apply || true
	sudo systemctl enable systemd-networkd
	sudo systemctl restart systemd-networkd

	echo "Obtaining IP address for the bridge via DHCP..."
	sudo dhcpcd -n $BRIDGE_NAME

	echo
	echo -e "${GREEN}Bridge setup complete.${RESET}"
	echo "Bridge name: $BRIDGE_NAME"
	echo "Bridged interfaces: $DEFAULT_IFACE ${WIFI_BRIDGE_IFACE:+and $WIFI_BRIDGE_IFACE} (plus bat0 if present)."
	echo "Check with:    ip a  (to see if $BRIDGE_NAME gets an IP via DHCP)"

fi

###############################################################################
# DONE
###############################################################################
echo -e "${GREEN}All steps are complete!${RESET}"
echo "Remember to: sudo systemctl enable --now hyved  (if desired)"
echo "You can also run: ip a  (to see interface statuses)"
