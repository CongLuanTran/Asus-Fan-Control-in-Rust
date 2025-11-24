install-all: stop-systemd install-fanctl install-systemd start-systemd

build:
	cargo build --release

install-fanctl: build
	sudo cp ./target/release/fanctl /usr/local/bin/

install-systemd:
	sudo cp fanctl.service /etc/systemd/system/

uninstall-systemd: stop-systemd
	sudo rm /etc/systemd/system/fanctl.service

start-systemd:
	sudo systemctl daemon-reload
	sudo systemctl start fanctl.service

stop-systemd:
	sudo systemctl stop fanctl.service

status-systemd:
	sudo systemctl status fanctl.service

