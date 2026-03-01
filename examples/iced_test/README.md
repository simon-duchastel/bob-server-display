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

# Edit the config to set your app path
nano ~/.config/sway/config
```

Key settings in the config:
- `seat * hide_cursor 1` - Hides cursor after 1ms (works without mouse)
- `exec swaymsg seat seat0 cursor move 9999 9999` - Moves cursor off-screen on startup
- `default_border none` - No window decorations
- `exec /path/to/iced-test` - Auto-starts your app

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

## FAQ

**Q: How do I hide the mouse cursor?**
With sway: The config uses two methods:
1. `seat * hide_cursor 1` - Hides cursor after 1ms of inactivity
2. `exec swaymsg seat seat0 cursor move 9999 9999` - Moves cursor off-screen on startup

If you still see a cursor, make sure the config path is correct and restart sway.

**Q: How do I make the display sleep after 10 minutes?**
Add to sway config:
```
exec swayidle -w timeout 600 'wlopm --off ALL' resume 'wlopm --on ALL'
```

**Q: How do I add touchscreen support?**
Touch works automatically via libinput in sway. For calibration:
```bash
sudo apt install libinput-tools
```

**Q: How do I exit the kiosk?**
With the provided config: Press `Alt+Shift+E` to exit sway.

**Q: Can I run multiple apps?**
Yes, add more `exec` lines to the sway config. But for a pure kiosk, stick with one app.

## Troubleshooting

**EGL errors on startup:**
```bash
export MESA_DEBUG=silent
export EGL_LOG_LEVEL=fatal
sway
```

**Sway won't start (permission denied):**
Make sure user is in `video` and `input` groups:
```bash
sudo usermod -a -G video,input $USER
# Log out and back in
```
