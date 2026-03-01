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

