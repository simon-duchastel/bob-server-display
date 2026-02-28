#!/bin/bash
set -e

echo "Installing Bob Server Display..."

# Create system user
echo "Creating bob-display user..."
if ! id -u bob-display &>/dev/null; then
    sudo useradd --system --no-create-home --shell /usr/sbin/nologin bob-display
    echo "User created successfully"
else
    echo "User already exists"
fi

# Create directories
echo "Creating directories..."
sudo mkdir -p /opt/bob-display
sudo mkdir -p /etc/bob-display
sudo mkdir -p /var/log/bob-display

# Set permissions
echo "Setting permissions..."
sudo chown -R bob-display:bob-display /opt/bob-display
sudo chown -R bob-display:bob-display /etc/bob-display
sudo chown -R bob-display:bob-display /var/log/bob-display

# Add user to video group
echo "Adding user to video group..."
sudo usermod -a -G video bob-display

# Create udev rules for DRM access
echo "Creating udev rules..."
sudo tee /etc/udev/rules.d/99-bob-display.rules > /dev/null << 'EOF'
# Allow bob-display user access to DRM devices
SUBSYSTEM=="drm", KERNEL=="card*", TAG+="uaccess", TAG+="seat"
SUBSYSTEM=="drm", KERNEL=="renderD*", TAG+="uaccess"
EOF

# Reload udev rules
sudo udevadm control --reload-rules

echo "Installation complete!"
echo ""
echo "Next steps:"
echo "1. Copy binary: sudo cp target/release/bob-server-display /opt/bob-display/"
echo "2. Copy config: sudo cp config/default.toml /etc/bob-display/config.toml"
echo "3. Copy service: sudo cp systemd/bob-display.service /etc/systemd/system/"
echo "4. Reload systemd: sudo systemctl daemon-reload"
echo "5. Enable service: sudo systemctl enable bob-display"
echo "6. Start service: sudo systemctl start bob-display"
echo ""
echo "Check status with: sudo systemctl status bob-display"