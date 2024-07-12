# NVIDIA_OC
NVIDIA_OC is a simple rust cli tool to overclock NVIDIA GPUs on Linux. This was made since the existing tools supported only X11 and not Wayland. This tool supports both X11 and Wayland.

Use something like `cron` to run this tool at startup with the desired overclock settings.

## Example usage
```bash
./nvidia_oc set --index 0 --power-limit 200000 --freq-offset 160 --mem-offset 850
```

## Run on startup
1. Grab the binary file from the [latest release](https://github.com/Dreaming-Codes/nvidia_oc/releases/)
2. Store it somewhere you won't accidentally delete it.
3. create `/etc/systemd/system/nvidia_oc.service` with the following content
```service
[Unit]
Description=NVIDIA Overclocking Service
After=network.target

[Service]
ExecStart=[folder where the bin is stored]/nvidia_oc set --index 0 --power-limit 200000 --freq-offset 160 --mem-offset 850
User=root
Restart=on-failure

[Install]
WantedBy=multi-user.target
```
> To do this you can run `sudo nano /etc/systemd/system/nvidia_oc.service` and paste the content above, then hit ctrl + x, Y and enter to confirm, enter again to confirm the path, done

4. Reload systemctl service by running `sudo systemctl daemon-reload`
5. Run `sudo systemctl enable --now nvidia_oc`
