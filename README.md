# Bob Display

GPU-accelerated kiosk display for my personal server.

## Running with Sway (Kiosk Mode)

**Sway** is a lightweight Wayland compositor.

### 1. Install Sway

```bash
sudo apt install sway swayidle
```

### 2. Configure Sway

Copy the provided config:

```bash
mkdir -p ~/.config/sway
cp sway-config ~/.config/sway/config
```

### 3. Build and Run

```bash
# Build the release binary
cargo build --release

# Run sway (it will auto-start bob-display)
sway

# Or with debug output
sway -d -V 2>&1 | tee sway.log
```

