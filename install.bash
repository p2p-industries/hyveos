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
	read -p "Continue? (y/n) " -n 1 -r

	echo

	if [[ "$REPLY" =~ ^[Yy]$ ]]; then
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
		# Open the link in the browser
		xdg-open https://docs.docker.com/engine/install/

		echo -e "Return to the terminal after installing Docker"
		read -p "Press any key to continue"
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
