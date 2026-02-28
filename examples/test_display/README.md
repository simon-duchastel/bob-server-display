# Test Display Example

A simple test program that displays "TEST TEST TEST" repeatedly across the screen to verify your display setup is working correctly.

## What it Does

- Clears the screen to black
- Prints "TEST TEST TEST " repeatedly across each line
- Fills the entire screen with text
- Adds a red border around the edges
- Runs at 60 FPS

## Building

```bash
# From the workspace root
cargo build -p test-display --release
```

Or from this directory:
```bash
cargo build --release
```

## Running

```bash
# Run directly (requires DRM device access)
sudo ./target/release/test-display

# Or with cargo
sudo cargo run -p test-display --release
```

## Configuration

Copy the config file to the system location:

```bash
sudo mkdir -p /etc/bob-display
sudo cp config/default.toml /etc/bob-display/config.toml
```

Or run from the example directory:
```bash
cd examples/test_display
sudo cargo run --release
```

## Expected Output

You should see:
- Black background
- Green "TEST TEST TEST " text filling the screen horizontally
- Multiple rows of text filling the screen vertically
- Red border around all edges

## Troubleshooting

**Permission denied on /dev/dri/card0:**
```bash
sudo usermod -a -G video $USER
# Log out and back in
```

**Black screen:**
- Check that HDMI cable is connected
- Verify display is powered on
- Try running from a TTY (not from X11/Wayland)

**Display not found:**
- Check available DRM devices: `ls -la /dev/dri/`
- Update config.toml with correct device path

## Stopping

Press Ctrl+C or send SIGTERM:
```bash
sudo pkill test-display
```