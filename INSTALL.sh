#!/usr/bin/env bash

# Use cargo to install the packages, the default installation is $Home/.cargo/bin
cargo install --path .
# Create a symlink to /usr/local/bin because systemd service should have static path
if [[ -z "$(find /usr/local/bin)" ]]; then
    sudo ln -s "$HOME/.cargo/bin/fanctl" /usr/local/bin
fi
# Copy the service file into the service file directory of systemd
sudo cp fanctl.service /etc/systemd/system

# Reload systemd daemon
sudo systemctl daemon-reload
# Enable the service
sudo systemctl enable fanctl
# Start it now
sudo systemctl start fanctl
