# Bob Server Display

A minimal, efficient display service for Raspberry Pi using DRM/KMS directly - no desktop environment required.
## Quick Start

### 1. Build the Project

```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/simon-duchastel/bob-server-display.git
cd bob-server-display
cargo build --release
```

### 2. Install

```bash
# Create system user and directories
sudo ./scripts/install.sh

# Copy binary
sudo cp target/release/bob-server-display /opt/bob-display/
sudo chmod +x /opt/bob-display/bob-server-display

# Copy default config
sudo cp config/default.toml /etc/bob-display/config.toml
sudo chmod 644 /etc/bob-display/config.toml

# Copy systemd service
sudo cp systemd/bob-display.service /etc/systemd/system/
sudo systemctl daemon-reload
```

### 3. Configure Display Permissions

Add the service user to the `video` group:

```bash
sudo usermod -a -G video bob-display
```

If using a Raspberry Pi with specific DRM permissions, you may need to adjust udev rules:

```bash
# Create udev rule for DRM access
sudo tee /etc/udev/rules.d/99-bob-display.rules << 'EOF'
SUBSYSTEM=="drm", KERNEL=="card*", TAG+="uaccess", TAG+="seat"
SUBSYSTEM=="drm", KERNEL=="renderD*", TAG+="uaccess"
EOF
sudo udevadm control --reload-rules
```

### 4. Start the Service

```bash
sudo systemctl enable bob-display
sudo systemctl start bob-display
```

Check status:

```bash
sudo systemctl status bob-display
sudo journalctl -u bob-display -f
```

## Configuration

Edit `/etc/bob-display/config.toml`:

```toml
# DRM device path (usually /dev/dri/card0 on Raspberry Pi)
drm_device = "/dev/dri/card0"

# Display mode (optional - auto-detect if not specified)
[mode]
width = 1920
height = 1080
refresh_rate = 60

# Colors (RGBA format)
background_color = [0, 0, 0, 255]    # Black
text_color = [255, 255, 255, 255]    # White

# Font settings
font_size = 24.0

# Performance
target_fps = 60
```

**Note:** Configuration changes require a service restart:

```bash
sudo systemctl restart bob-display
```
