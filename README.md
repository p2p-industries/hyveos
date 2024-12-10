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

Now you can install `hyved` (the HyveOS daemon):

```bash
sudo apt install -y hyved
```

Before you can run `hyved`, you need to configure it.
You can do this by editing `/etc/hyved/config.toml`, which should at least configure the network interfaces (using the `interfaces` key) and which of these interfaces is the B.A.T.M.A.N.-adv virtual interface (using the `batman-interface` key).
In the following example, `wlan0` is the wireless interface used for the B.A.T.M.A.N.-adv mesh, and `bat0` is the B.A.T.M.A.N.-adv virtual interface:

```toml
interfaces = ["bat0", "wlan0"]
batman-interface = "bat0"
```

These interfaces are assumed for the rest of this installation guide.

Before running `hyved` you need to setup its prerequisites. The easiest way to do this is to use the provided systemd services:

```bash
sudo systemctl start docker
sudo systemctl start wpa_supplicant@wlan0
sudo systemctl start hyveos-batman@wlan0
sudo systemctl start batman-neighbours-daemon
```

Now you can start `hyved`:

```bash
sudo systemctl start hyved
```

To run it at boot, you can enable the systemd services:

```bash
sudo systemctl enable --now docker
sudo systemctl enable --now wpa_supplicant@wlan0
sudo systemctl enable --now hyveos-batman@wlan0
sudo systemctl enable --now batman-neighbours-daemon
sudo systemctl enable --now hyved
```
