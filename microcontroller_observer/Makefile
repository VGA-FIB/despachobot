.PHONY: build
build:
	arduino-cli compile -b arduino:megaavr:nona4809

.PHONY: upload
upload: build
	arduino-cli upload -b arduino:megaavr:nona4809 -p /dev/ttyACM0

.PHONY: monitor
monitor: upload
	arduino-cli monitor -p /dev/ttyACM0 -c 115200
