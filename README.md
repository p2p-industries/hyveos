# ![P2P Industries Logo](logo.png)
[![Rust](https://github.com/p2p-industries/mvp/actions/workflows/rust.yml/badge.svg)](https://github.com/p2p-industries/mvp/actions/workflows/rust.yml)

## P2P Industries builds the next generation of industrial automation.

## Installation

### Debian/Ubuntu

Add the P2P Industries APT repo and install the prerequisites (B.A.T.M.A.N.-adv and docker):

```bash
wget -qO - https://apt.p2p.industries/key.gpg | sudo gpg --dearmor --yes -o /etc/apt/trusted.gpg.d/p2p-industries.gpg
echo "deb https://apt.p2p.industries /" | sudo tee /etc/apt/sources.list.d/p2p-industries.list
sudo apt update && sudo apt upgrade -y
sudo apt install -y batctl
curl -sSL https://get.docker.com | sh
```

Now you can install the P2P Industries stack:

```bash
sudo apt install -y p2p-industries-stack
```

To run it at boot, you need to enable a few services:

```bash
sudo systemctl enable --now docker
sudo systemctl enable --now batman-neighbours-daemon
sudo systemctl enable --now wpa_supplicant@wlan0
sudo systemctl enable --now p2p-industries-batman@wlan0
sudo systemctl enable --now p2p-industries-stack
```
