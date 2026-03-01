# Iced GUI Example

Simple GPU-accelerated GUI example using Iced with wgpu backend.

## Running

This example requires a Wayland or X11 display server. For kiosk deployments, use `cage`:

```bash
# Install cage (single-app Wayland compositor)
sudo apt install cage

# Run the example
cage -- cargo run --example iced_test --release

# Or run the built binary
cage -- ./target/release/iced-test
```

## Kiosk Deployment

For a headless Pi kiosk that starts automatically:

```bash
# Run via systemd service
sudo systemd-run --unit=bob-display --property=Type=simple --collect --wait \
    cage -- ./target/release/iced-test
```

## FAQ

**Q: How do I hide the mouse cursor?**
```bash
# Cage automatically hides cursor after inactivity
# For immediate hide, use unclutter or add to your service:
export WLR_NO_HARDWARE_CURSORS=1
```

**Q: How do I add touchscreen support?**
Touch should work automatically via libinput. For calibration:
```bash
sudo apt install libinput-tools
```

**Q: How do I make the display sleep after 10 minutes?**
Use `swayidle` with `cage`:
```bash
sudo apt install swayidle
cage -- swayidle -w timeout 600 'wlopm --off ALL' resume 'wlopm --on ALL' &
./target/release/iced-test
```

**Q: How do I run it 24/7 without a terminal?**
Create a systemd service - see the main README for service setup examples.
