# ![P2P Industries Logo](logo.png)

[![Crates.io Version](https://img.shields.io/crates/v/hyveos-sdk)](https://crates.io/crates/hyveos-sdk)
[![NPM Version](https://img.shields.io/npm/v/%40hyveos%2Fsdk)](https://www.npmjs.com/package/@hyveos/sdk)
[![JSR](https://jsr.io/badges/@hyveos/sdk)](https://jsr.io/@hyveos/sdk)
[![PyPI - Version](https://img.shields.io/pypi/v/hyveos-sdk)](https://pypi.org/project/hyveos-sdk/)

## P2P Industries builds the first decentralized robot communication system

## [Documentation](https://docs.p2p.industries)

[https://docs.p2p.industries](https://docs.p2p.industries).

We've tried to provide comprehensive documentation for everything that you can do with hyveOS!

## Example Use Cases

While the provided abstractions are universally useful, hyveOS is ideally suited for robot swarms.

### Live Video to first Responders

Today many search and rescue operators already use drones to save lives. With hyveOS's [File Transfer](https://docs.p2p.industries/sdk/file_transfer/) the video feed can be
broadcastet to countless on-the-ground operators even while the cellular network is down.

### Warehouse

Any time robots come together, like in a warehouse, communication is key. Instead of relying on a central server that distributes tasks
or provides information about roles (who can do what), with hyveOS the robots can exchange that information between each other.

Think of a warehouse where robots move individual parcels, but sometimes they need to come together to carry a pallet or a bit item.

With hyveOS one robot that took up the task can publish a message in the `carry-stuff` topic of our [Pub-Sub](https://docs.p2p.industries/sdk/pub_sub/), asking robots that are in [proximity](https://docs.p2p.industries/sdk/discovery/) for assitance.
They can [respond to the request](https://docs.p2p.industries/sdk/request_response/) and once they are enough, they can proceed together.

### Infrastructure Interaction

Think of a service robot that wants to deliver a coffee to you, while you are sitting at your desk.

With hyveOS the service robot can directly talk to your computer (or a website where you ordered the coffee), the coffee machine, the doors in the way
and the elevator that takes it to the right floor. Made possible by our [Key Value Store](https://docs.p2p.industries/sdk/dht/) (find a coffee machine) and [request-response](https://docs.p2p.industries/sdk/request_response/) (open the door for me please).

## Installation

## Install in One Line

> **This script will make changes to the network configuration of your device.**
> It's possible that you will loose access to the device during the installation. If that happens it's usually safe to log back into the device and run the script again.
> Don't use this on devices you don't have physical access to, contains important data or is in production.
>
> The best configuration during **setup** is an ethernet cable plugged into the device over which you ssh into the device and a wifi chip on the device.
> We tested the script on Ubuntu Server 24.04 LTS and Raspberry Pi 5. Different devices and somewhat recent Debian derivates should work as well.

```bash
bash <(curl -ssL https://install.p2p.industries)
```

This will run our installation script where everything is preconfigured for you.
If you want to or need to have some more control, keep to the following 4 steps:

## Install through APT Repository

1. Set up hyve `apt` repository:
   ```bash
   wget -qO - https://apt.p2p.industries/key.gpg | sudo gpg --dearmor --yes -o /etc/apt/trusted.gpg.d/p2p-industries.gpg
   echo "deb https://apt.p2p.industries /" | sudo tee /etc/apt/sources.list.d/p2p-industries.list
   sudo apt update && sudo apt upgrade -y
   ```

2. Install `docker` (If you need to):
   ```bash
   curl -sSL https://get.docker.com | sh
   ```

3. Install `B.A.T.M.A.N.-adv`:
   ```bash
   sudo apt install -y batctl
   ```

4. Install `hyved` and `hyvectl`:
   ```bash
   sudo apt install -y hyved hyvectl
   ```

### Choose a WiFi network interface for the mesh

You need to choose a WiFi network interface for the mesh.
This interface will be configured using `wpa_supplicant`, so you have to ensure that it isn't managed by any other networking service like `Netplan` or `NetworkManager` before continuing.

> On a recent Raspberry Pi, the WiFi interface is usually called `wlan0`.
>
> In the rest of this guide, we will assume that `wlan0` is the WiFi interface you want to use for the mesh.
> Otherwise, replace `wlan0` with the name of your WiFi interface in the following steps.

<details>
<summary>How to stop `Netplan` from managing the `wlan0` interface on a default Raspberry Pi Ubuntu 24.04 installation</summary>
On a default Raspberry Pi Ubuntu 24.04 installation, installed using the Raspberry Pi Imager, `Netplan` is configured by `cloud-init` to manage the `wlan0` interface.
To stop `Netplan` from managing the interface, you should be able to follow these steps:

1. Make sure, that you are either working on the device directly (e.g. with a keyboard and monitor connected to the Pi) or that you are connected over SSH using the ethernet port.

2. Check if `cloud-init` is enabled:
   ```bash
   sudo systemctl status cloud-init
   ```

   If it is enabled, you should see something like `Active: active` in the output.
   In that case, you can disable it permanently by creating an empty file at `/etc/cloud/cloud-init.disabled`:

   ```bash
   sudo touch /etc/cloud/cloud-init.disabled
   ```

3. Check, which network configuration files are present:
   ```bash
   ls /etc/netplan
   ```

   If `cloud-init` was enabled, you should see a file called something like `50-cloud-init.yaml`. Otherwise, other configuration files might be present.
   Check, which of these files contains the configuration for the `wlan0` interface:

   ```bash
   cat /etc/netplan/50-cloud-init.yaml
   # maybe cat other files as well, until you find the one with the `wlan0` configuration
   ```

   The file should look somewhat like this:

   ```yaml
   network:
     version: 2
     wifis:
       renderer: networkd
       wlan0:
         access-points:
           SOME_SSID:
             password: SOME_PASSWORD_HASH
         dhcp4: true
         optional: true
     ethernets:
       eth0:
         dhcp4: true
   ```

4. Remove the configuration for the `wlan0` interface from the file:
   ```bash
   sudo nano /etc/netplan/50-cloud-init.yaml
   ```

   Remove the section for the `wlan0` interface. The example above would look like this after removing the `wlan0` section:

   ```yaml
   network:
     version: 2
     ethernets:
       eth0:
         dhcp4: true
   ```

   If none of the configuration file you find in step 3 contains an ethernet configuration, but you are connected over ethernet, you might need to add an ethernet configuration like the one above, to keep the ethernet connection working.

5. Apply the changes:
   ```bash
   sudo netplan apply
   ```
   If you are connected over SSH, you might lose the connection at this point. If you do, just try to reconnect after a few seconds.
   If reconnecting doesn't work, you might need to restart the device.

</details>

### Configure `hyved` before Running

Before you can run `hyved`, you need to configure it.
You can do this by editing `/etc/hyved/config.toml`,
which should at least configure the network interfaces (using the interfaces key).

In the following example, `wlan0` is the wireless interface used for
the **B.A.T.M.A.N.-adv** mesh, and `bat0` is the **B.A.T.M.A.N.-adv** virtual interface:

```toml
interfaces = ["bat0", "wlan0"]
batman-interface = "bat0"
```

> You should see other possible configuration options in the comments in the config file.

Before running hyved you need to setup its prerequisites.
The easiest way to do this is to use the provided systemd services:

```bash
sudo systemctl enable --now docker
sudo systemctl enable --now wpa_supplicant@wlan0
sudo systemctl enable --now hyveos-batman@wlan0
sudo systemctl enable --now batman-neighbours-daemon
```

### Starting `hyved`

```bash
sudo systemctl start hyved
```

Now `hyved` is running on your machine.

### Running `hyved` at Boot

If you want to run `hyved` at boot, enable the `systemd` service:

```bash
sudo systemctl enable --now hyved
```

### Adding your user to the `hyveos` group

To run `hyvectl` without `sudo`, you need to add your user to the `hyveos` group:

```bash
sudo usermod -aG hyveos $USER
```

## Verify the installation

Verify your installation by running

```bash
hyvectl whoami
```

You should see something like

```bash
🤖 You are { 12D3KooWJtoSLKL5H7GJsx9rExxwL4ZdckgX2ENagSHDrMSdxt7A }
```

If it succeeds, that's it! Your machine is ready to join multi-node swarms!

Continue with the [Quickstart](https://docs.p2p.industries/concepts/quickstart) or learn about [hyveOS concepts](https://docs.p2p.industries/concepts/concepts)
