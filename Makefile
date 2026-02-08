install-all: start-systemd

build:
	cargo build --release

install-fanctl: build stop-systemd
	sudo cp ./target/release/fanctl /usr/local/bin/

install-systemd: install-fanctl
	sudo cp fanctl.service /etc/systemd/system/

uninstall-systemd: stop-systemd
	sudo rm /etc/systemd/system/fanctl.service

start-systemd: install-systemd
	sudo systemctl daemon-reload
	sudo systemctl start fanctl.service

stop-systemd:
	sudo systemctl stop fanctl.service || true

status-systemd:
	sudo systemctl status fanctl.service

