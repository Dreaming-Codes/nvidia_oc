# NVIDIA_OC

NVIDIA_OC is a simple Rust CLI tool designed to overclock NVIDIA GPUs on Linux. This tool was developed to support both X11 and Wayland environments, addressing a gap in existing overclocking tools that only support X11.

## Example Usage

To set the overclock parameters for your NVIDIA GPU, use the following command:

```bash
./nvidia_oc set --index 0 --power-limit 200000 --freq-offset 160 --mem-offset 850
```

## Run on Startup

To ensure NVIDIA_OC runs on startup, follow these steps:

1. Download the binary file from the [latest release](https://github.com/Dreaming-Codes/nvidia_oc/releases/).
2. Store the binary file in a secure location.
3. Create a systemd service file at `/etc/systemd/system/nvidia_oc.service` with the following content:

```service
[Unit]
Description=NVIDIA Overclocking Service
After=network.target

[Service]
ExecStart=[path_to_binary]/nvidia_oc set --index 0 --power-limit 200000 --freq-offset 160 --mem-offset 850
User=root
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

Replace `[path_to_binary]` with the actual path where the binary is stored.

To create this file, you can run:

```bash
sudo nano /etc/systemd/system/nvidia_oc.service
```

Paste the content above, then press `Ctrl + X`, `Y` to confirm saving, and `Enter` to confirm the file path.

4. Reload the systemd manager configuration:

```bash
sudo systemctl daemon-reload
```

5. Enable and start the service immediately:

```bash
sudo systemctl enable --now nvidia_oc
```

## Funding

This application is completely free, and I do not earn any money from your usage of it. If you would like to support my work, donations via PayPal or GitHub Sponsors are greatly appreciated.

- **PayPal:** [Donate via PayPal](https://paypal.me/dreamingcodes)
- **GitHub Sponsors:** [Sponsor on GitHub](https://github.com/sponsors/Dreaming-Codes)

Thank you for your support!
