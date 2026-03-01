# Iced GUI Example

Simple GPU-accelerated GUI example using Iced with wgpu backend.

## Running with Sway (Recommended for Kiosk)

**Sway** is a lightweight Wayland compositor.

### 1. Install Sway

```bash
sudo apt install sway swayidle
```

### 2. Configure Sway

Copy the provided config:

```bash
mkdir -p ~/.config/sway
cp examples/iced_test/sway-config ~/.config/sway/config

### 3. Run Sway

```bash
# Run sway (it will auto-start your app)
sway

# Or with debug output
sway -d -V 2>&1 | tee sway.log
```

### 4. Auto-start on Boot (systemd)

Create `/etc/systemd/system/bob-display.service`:

```ini
[Unit]
Description=Bob Display Kiosk
After=systemd-user-sessions.service

[Service]
Type=simple
User=pi
Environment="WLR_BACKENDS=drm"
Environment="XDG_RUNTIME_DIR=/run/user/1000"
ExecStart=/usr/bin/sway
Restart=always
RestartSec=5

[Install]
WantedBy=graphical.target
```

Enable and start:
```bash
sudo systemctl enable bob-display
sudo systemctl start bob-display
```
