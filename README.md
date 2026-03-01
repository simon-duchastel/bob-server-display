# Bob Server Display

A minimal, efficient display service for Raspberry Pi using DRM/KMS directly - no X11, no Wayland, no desktop environment required.

## Features

- **Direct DRM/KMS access** - Renders directly to the framebuffer without display server overhead
- **Minimal resource usage** - Optimized for embedded systems
- **Simple bitmap font** - Self-contained with no external font dependencies
- **Systemd integration** - Ready to run as a system service
- **Hot-restart configuration** - Restart service to reload config (no SIGHUP complexity)
- **60 FPS rendering** - Smooth display updates

## Requirements

- Raspberry Pi (2, 3, 4, or 5)
- HDMI display connected
- No X11 or Wayland running (pure console/TTY mode)
- Rust 1.70+ (for building)
- Build dependencies: `libgbm-dev`, `libdrm-dev`

### Install Build Dependencies (Debian/Ubuntu)

```bash
sudo apt-get update
sudo apt-get install -y libgbm-dev libdrm-dev
```

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

## Development

### Project Structure

```
src/
├── main.rs      # Entry point, signal handling
├── display.rs   # DRM/KMS initialization & buffer management
├── render.rs    # Drawing primitives (text, shapes)
└── config.rs    # Configuration loading

config/
└── default.toml # Default configuration template

systemd/
└── bob-display.service # Systemd service file

scripts/
└── install.sh   # Installation script
```

### Running Locally

For development on a Linux machine with DRM support:

```bash
# Make sure you're in the video group
sudo usermod -a -G video $USER

# Run directly (requires DRM master access)
cargo run --release
```

**Note:** Running directly from TTY usually works best. From an X11/Wayland session, you may need to stop your display server first.

### Building for Raspberry Pi

Cross-compile for ARM:

```bash
# Add ARM target
rustup target add aarch64-unknown-linux-gnu

# Install cross-compilation tools
sudo apt-get install gcc-aarch64-linux-gnu

# Build
CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
  cargo build --release --target aarch64-unknown-linux-gnu
```

## Architecture

### DRM/KMS Flow

1. Open `/dev/dri/card0` with read/write permissions
2. Query connected displays and available modes
3. Select or create framebuffer
4. Use GBM (Generic Buffer Management) for GPU buffers
5. Render to buffer using CPU (can be enhanced with GPU acceleration)
6. Atomic commit to display buffer

### Rendering

- **Double buffering** - Not yet implemented (rendering directly)
- **Bitmap font** - 5x7 pixel font embedded in binary
- **Color format** - RGBA32 (XRGB8888 in DRM terms)

### Performance

- ~60 FPS target (configurable)
- CPU-based rendering (GPU acceleration possible future enhancement)
- Minimal memory footprint (~10-20MB for typical 1080p framebuffer)

## Troubleshooting

### Service fails to start

```bash
# Check logs
sudo journalctl -u bob-display -n 50

# Verify DRM device exists and is accessible
ls -la /dev/dri/
groups bob-display  # Should include 'video'
```

### Permission denied on /dev/dri/card0

```bash
# Fix permissions
sudo chmod 666 /dev/dri/card0  # Temporary fix

# Or better, add proper udev rules (see installation section)
```

### Black screen

1. Verify display is connected and powered on
2. Check HDMI cable
3. Test with: `kmstest` or `modetest` from `libdrm-tests`
4. Verify correct DRM device in config

### Display corruption

- May indicate memory/buffer issues
- Check dmesg for GPU errors: `sudo dmesg | grep -i drm`

## Future Enhancements

- [ ] Double buffering for tear-free rendering
- [ ] Hardware acceleration via OpenGL ES
- [ ] Touch/keyboard input support
- [ ] Dynamic configuration reload
- [ ] HTTP API for external control
- [ ] Image loading and caching
- [ ] Animation support
- [ ] Multiple display support

## License

MIT License - See LICENSE file for details

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test on actual Raspberry Pi hardware
5. Submit a pull request

## Acknowledgments

- Built with [drm-rs](https://github.com/Smithay/drm-rs) for DRM/KMS bindings
- Uses [GBM](https://github.com/Smithay/gbm.rs) for buffer management
- Inspired by the minimalism of embedded display systems