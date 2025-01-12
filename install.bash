#!/usr/bin/env bash

# Set color to green
GREEN="\033[0;32m"

# Set color to red
RED="\033[0;31m"

# Set color to yellow

YELLOW="\033[0;33m"

# Reset color

RESET="\033[0m"

int_handler() {
	echo -e "\n${YELLOW}Interrupted.${RESET}"
	# Kill the parent process of the script.
	kill $PPID
	exit 1
}
trap 'int_handler' INT

echo -e "${GREEN}Welcome to the installation script!${RESET}"

function continue_installation() {
	# Default is yes and if the users presses enter, it will continue with the installation.
	read -p "Continue (Y/n)? " -n 1 -r

	echo

	# If the user presses enter, it will continue with the installation.

	if [[ -z "$REPLY" ]]; then
		echo -e "${GREEN}Continuing installation${RESET}"
	elif [[ "$REPLY" =~ ^[Yy]$ ]]; then
		echo -e "${GREEN}Continuing installation${RESET}"
	else
		echo -e "${RED}Exiting installation${RESET}"
		exit 1
	fi
}

# Detect the OS (Linux or MacOS)

if [[ "$(uname)" == "Darwin" ]]; then
	echo -e "${RED}This script is not supported on MacOS${RESET}"
	exit 1
fi

if [[ "$(uname)" == "Linux" ]]; then
	echo -e "${GREEN}Linux detected${RESET}"
fi

# Check if distro is Ubuntu or Raspberry Pi OS (Raspbian)

if [ -f /etc/os-release ]; then
	. /etc/os-release
	OS=$NAME
	VER=$VERSION_ID

	if [[ "$OS" == "Raspbian GNU/Linux" ]]; then
		echo -e "${GREEN}Raspberry Pi OS detected${RESET}"
	elif [[ "$OS" == "Ubuntu" ]]; then
		echo -e "${GREEN}Ubuntu detected${RESET}"

		# Check if version is 22.04 or 24.04
		if [[ "$VER" == "22.04" ]]; then
			echo -e "${GREEN}Ubuntu 22.04 detected${RESET}"
		elif [[ "$VER" == "24.04" ]]; then
			echo -e "${GREEN}Ubuntu 24.04 detected${RESET}"
		else
			echo -e "${RED}Unsupported Ubuntu version${RESET}"
			echo -e "${RED}Proceed at your own risk?${RESET}"

			continue_installation
		fi

	else
		echo "${RED}Unsupported OS${RESET}"
		echo "${RED}Proceed at your own risk?${RESET}"

		continue_installation
	fi
fi

# Check if architecture is aarch64 or x86_64

ARCH=$(uname -m)

if [[ "$ARCH" == "aarch64" ]]; then
	echo -e "${GREEN}ARM64 detected${RESET}"
elif [[ "$ARCH" == "x86_64" ]]; then
	echo -e "${GREEN}x86_64 detected${RESET}"
else
	echo -e "${RED}Unsupported architecture${RESET}"
	echo -e "${YELLOW}Supported architectures are aarch64 and x86_64${RESET}"
	echo -e "${RED}Proceed at your own risk?${RESET}"

	continue_installation
fi

# Install dependencies

read -p "Do you want to install dependencies for this script (wget, curl, gpg, jq)? (y/n) " -n 1 -r

echo

if [[ "$REPLY" =~ ^[Yy]$ ]]; then
	echo -e "${GREEN}Installing dependencies${RESET}"
	sudo apt update
	sudo apt install wget curl gpg jq
else
	echo -e "${RED}Skipping installation of dependencies${RESET}"
	exit 1
fi

# Check if docker is installed (docker --version)

if [[ -x "$(command -v docker)" ]]; then
	echo -e "${GREEN}Docker is already installed and running${RESET}"
else
	# Install Docker

	# Prompt the user on how to install docker.
	# 1. Install Docker themselves (manual) (recommended). If the user chooses that open: https://docs.docker.com/engine/install/
	# 2. Install Docker using the script (automatic) (not recommended). If the user chooses that, run the script below.
	# https://docs.docker.com/engine/install/ubuntu/#install-using-the-convenience-script
	# curl -sSL https://get.docker.com | sh
	# 3. Exit the installation.

	echo -e "${RED}Docker is not installed${RESET}"
	echo -e "${YELLOW}You have three options:${RESET}"
	echo -e "${YELLOW}1. Install Docker yourself (link with documentation will be provided)${RESET}"
	echo -e "${YELLOW}2. Install Docker using docker convenience script. (easy + potentially unsafe)${RESET}"
	echo -e "${YELLOW}3. Exit the installation${RESET}"

	read -p "Choose an option (1/2/3): " -n 1 -r
	echo

	if [[ "$REPLY" == "1" ]]; then
		echo -e "${YELLOW}Please install Docker manually by visiting the following link: https://docs.docker.com/engine/install/${RESET}"
		# Open the link in the browser but first check if xdg-open is installed

		if [[ -x "$(command -v xdg-open)" ]]; then
			xdg-open https://docs.docker.com/engine/install/
		else
			echo -e "${YELLOW}Please visit the following link to install Docker: https://docs.docker.com/engine/install/${RESET}"
		fi

		echo -e "Return to the terminal after installing Docker"
		read -p "Press enter key to continue"
	elif [[ "$REPLY" == "2" ]]; then
		echo -e "${YELLOW}Installing Docker using the convenience script${RESET}"
		curl -sSL https://get.docker.com | sh || (echo -e "${RED}Error encountered while installing Docker${RESET}" && exit 1)
		echo "${GREEN}Docker installed successfully${RESET}"
	elif [[ "$REPLY" == "3" ]]; then
		echo -e "${RED}Exiting installation${RESET}"
		exit 1
	else
		echo -e "${RED}Invalid option${RESET}"
		echo -e "${RED}Exiting installation${RESET}"
		exit 1
	fi
fi

read -p "Do you want to install mesh support? (y/n) " -n 1 -r
echo

if [[ "$REPLY" =~ ^[Yy]$ ]]; then
	MESH_SUPPORT=false
	echo -e "${GREEN}Installing mesh support${RESET}"

	# Check if batctl is installed (batctl --version)

	if [[ -x "$(command -v batctl)" ]]; then
		echo -e "${GREEN}Batctl is already installed${RESET}"
	else
		# Install batctl

		sudo apt install -y batctl
	fi

	# Check if batman-adv is installed (modinfo batman-adv)

	if [[ -x "$(command -v modinfo)" ]]; then
		if modinfo batman-adv &>/dev/null; then
			echo -e "${GREEN}Batman-adv is already installed${RESET}"
			MESH_SUPPORT=true
		else
			# Install batman-adv
			echo -e "${YELLOW}The batman-adv kernel module is not installed. Very often it can be installed by installing the extra linux kernel packages.${RESET}"
			read -p "Do you want to install the batman-adv kernel module? (y/n) via linux-modules-extra-$(uname -r) " -n 1 -r
			if [[ "$REPLY" =~ ^[Yy]$ ]]; then
				sudo apt install linux-modules-extra-$(uname -r)

				if modinfo batman-adv &>/dev/null; then
					echo -e "${GREEN}Batman-adv installed successfully${RESET}"
					MESH_SUPPORT=true

				else
					echo -e "${RED}Failed to install batman-adv${RESET}"
					echo -e "${RED}Proceeding without mesh support${RESET}"
				fi
			fi
		fi

		# If mesh support is enabled (MESH_SUPPORT=true), activate the batman-adv kernel module

		if [[ "$MESH_SUPPORT" == true ]]; then
			# Check if batman-adv is loaded (lsmod | grep batman-adv)

			if lsmod | grep batman-adv &>/dev/null; then
				echo -e "${GREEN}Batman-adv is already loaded${RESET}"
			else
				# Load batman-adv

				sudo modprobe batman-adv

				# Also load batman-adv at boot

				echo -e "${GREEN}Enabling batman-adv to run at boot${RESET}"
				sudo sh -c "echo batman-adv >> /etc/modules"
			fi
		fi

	fi
else
	echo -e "${RED}Skipping mesh support${RESET}"
fi

# Checking if hyved is installed

if [[ -x "$(command -v hyved)" ]]; then
	echo -e "${GREEN}Hyved is already installed${RESET}"
else
	echo -e "${RED}Hyved is not installed${RESET}"
	echo -e "${YELLOW}Installing it now ${RESET}"
	echo "${GREEN}Adding our GPG key${RESET}"
	echo "deb https://apt.p2p.industries /" | sudo tee /etc/apt/sources.list.d/p2p-industries.list
	wget -qO - https://apt.p2p.industries/key.gpg | sudo gpg --dearmor --yes -o /etc/apt/trusted.gpg.d/p2p-industries.gpg
	sudo apt update
	sudo apt install hyved
fi

# TODO: Configure interfaces with hyvectl

echo -e "${RED}WARNING: If you choose yes in the following step, the wifi interface over which you might be connected will change its config and you won't be able to connect to it conventionally anymore. Please make sure you are connected via an alternative interface or ok with losing access. You can still access the computer with an attached monitor and keyboard or via other devices in the network.${RESET}"
read -p "Do you want to configure the interfaces now? (y/n) " -n 1 -r

echo

if [[ "$REPLY" =~ ^[Yy]$ ]]; then
	echo -e "${GREEN}Configuring interfaces${RESET}"
	interface=$(hyvectl init-config --json | jq -r '.selected-interface')

	echo "Configuring interface $interface"

	sudo systemctl enable --now wpa_supplicant@$interface
	sudo systemctl enable --now hyveos-batman@$interface
fi

echo -e "${GREEN}Enabling docker and the batman-neighbours-daemon to run at boot.${RESET}"

continue_installation

sudo systemctl enable --now docker
sudo systemctl enable --now batman-neighbours-daemon

echo -e "${GREEN}Installation complete${RESET}"

echo "To start the hyved service, run: sudo systemctl start hyved"
echo "To start the hyved service at boot, run: sudo systemctl enable --now hyved"

### Ask the user if they want to setup this node as a bridge (between the mesh and the internet)

echo -e "${RED}WARNING: If you choose yes in the following step, the wifi interface over which you might be connected will change its config and you won't be able to connect to it conventionally anymore. Please make sure you are connected via an alternative interface or ok with losing access. You can still access the computer with an attached monitor and keyboard or via other devices in the network.${RESET}"
read -p "Do you want to setup this node as a bridge? (y/n) " -n 1 -r

if [[ "$REPLY" =~ ^[Yy]$ ]]; then

	set -e

	###############################################################################
	# 1. Check for Ubuntu + netplan, prompt user to remove interface from netplan
	###############################################################################
	# We assume "Debian derivatives only," so let's just check if /etc/os-release
	# contains the word "Ubuntu". Then check if netplan is installed and in use.

	if grep -qi "Ubuntu" /etc/os-release; then
		# Check if netplan YAML files exist
		if ls /etc/netplan/*.yaml &>/dev/null; then
			echo "Detected Ubuntu system with netplan files."
			echo "We are going to open each netplan file in an editor."
			echo "Please remove or comment out the configuration for the interface you plan to manage with systemd-networkd."

			# Loop over each YAML file in /etc/netplan
			for npfile in /etc/netplan/*.yaml; do
				echo "Opening $npfile in nano..."
				# Inform user which interface(s) might be in that file
				echo "Current netplan file contents (for reference):"
				echo "-------------------------------------------------"
				cat "$npfile"
				echo "-------------------------------------------------"
				echo "Press ENTER to open $npfile in nano, remove the interface config, then save/exit."
				read -r
				nano "$npfile"
			done

			echo "Netplan configuration might have been updated."
			echo "Applying netplan changes..."
			netplan apply || true
			echo "Done applying netplan."
			echo
		fi
	fi

	###############################################################################
	# 2. Install needed packages
	###############################################################################
	echo "Installing required packages..."
	# systemd-networkd is typically pre-installed on Ubuntu, but let's ensure:
	apt-get update
	apt-get install -y systemd-networkd curl

	# If you want to enable/disable systemd-networkd over NetworkManager, you can do so:
	# systemctl enable systemd-networkd
	# systemctl disable NetworkManager

	###############################################################################
	# 3. Detect an active Ethernet interface with Internet
	###############################################################################
	# We will try to find a default route interface that is:
	# - An Ethernet interface (has link/ether).
	# - Actually "UP".
	# - Successfully allows 'curl' to a known site (e.g., google.com).

	echo "Detecting an active Ethernet interface with Internet connectivity..."

	DEFAULT_IFACE=""
	# Use 'ip route show default' to find the interface used for default route
	CANDIDATE_IFACE=$(ip route show default 2>/dev/null | awk '/default/ {print $5; exit}')

	if [ -n "$CANDIDATE_IFACE" ]; then
		# Check if it's an Ethernet interface
		if ip link show "$CANDIDATE_IFACE" | grep -q "link/ether"; then
			echo "Candidate default route interface $CANDIDATE_IFACE is Ethernet. Testing connectivity..."
			# Try to use curl on that interface
			if curl --interface "$CANDIDATE_IFACE" -s --max-time 5 https://ifconfig.me >/dev/null; then
				DEFAULT_IFACE="$CANDIDATE_IFACE"
			else
				echo "Could not verify Internet connectivity on $CANDIDATE_IFACE with curl."
			fi
		else
			echo "Default route interface $CANDIDATE_IFACE is not Ethernet."
		fi
	else
		echo "No default route interface found!"
	fi

	# If no good candidate was found, attempt scanning all 'eth*' or 'en*' interfaces
	if [ -z "$DEFAULT_IFACE" ]; then
		for IFACE in $(ls /sys/class/net | grep -E '^eth|^en'); do
			# Is interface up?
			if ip link show "$IFACE" | grep -q "state UP"; then
				# Try connectivity test
				if curl --interface "$IFACE" -s --max-time 5 http://ifconfig.me >/dev/null; then
					DEFAULT_IFACE="$IFACE"
					break
				fi
			fi
		done
	fi

	if [ -z "$DEFAULT_IFACE" ]; then
		echo "ERROR: No Ethernet interface with verified Internet connectivity was found."
		echo "Please configure your network manually or ensure that an interface is up and can reach the Internet."
		exit 1
	fi

	echo "Using interface '$DEFAULT_IFACE' as the primary (Ethernet + Internet) interface."

	###############################################################################
	# 4. Create systemd-networkd configuration files
	###############################################################################
	# We'll assume you want a bridge called br0 that:
	# - Has DHCPv4 on br0
	# - enslaves DEFAULT_IFACE in the bridge
	# - Might also enslave bat0 (if you have batman-adv)
	# Adjust or remove bat0 part if not desired.

	# Ask the user for the bridge name
	read -p "Enter the name of the bridge (default: br0): " BRIDGE_NAME
	echo "Using bridge name: ${BRIDGE_NAME:-br0}"
	export BRIDGE_NAME=${BRIDGE_NAME:-br0}

	mkdir -p /etc/systemd/network

	# 4a. The .netdev for the bridge
	cat <<EOF >/etc/systemd/network/10-br0.netdev
[NetDev]
Name=$BRIDGE_NAME
Kind=bridge
EOF

	# 4b. The .network for the bridge itself
	cat <<EOF >/etc/systemd/network/10-br0.network
[Match]
Name=$BRIDGE_NAME

[Network]
DHCP=ipv4
EOF

	# 4c. The .network for the default Ethernet interface
	cat <<EOF >/etc/systemd/network/20-$DEFAULT_IFACE.network
[Match]
Name=$DEFAULT_IFACE

[Network]
DHCP=no
Bridge=$BRIDGE_NAME
EOF

	# 4d. Optionally, if you have a batman-adv interface named bat0, also remove DHCP and enslave it:
	if ip link show bat0 &>/dev/null; then
		cat <<EOF >/etc/systemd/network/30-bat0.network
[Match]
Name=bat0

[Network]
DHCP=no
Bridge=$BRIDGE_NAME
EOF
	fi

	echo "Systemd-networkd configuration files created in /etc/systemd/network/."

	###############################################################################
	# 5. Enable and restart systemd-networkd
	###############################################################################
	systemctl enable systemd-networkd
	systemctl restart systemd-networkd

	echo "Done! Your bridge (br0) should now get an IP via DHCP, with $DEFAULT_IFACE enslaved."
	echo "If bat0 exists, it is also enslaved with no DHCP configured on it."
fi
