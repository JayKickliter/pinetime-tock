#!/bin/sh

while ! nc -t -z 127.0.0.1 2331; do
	sleep 0.1
done
cgdb -d arm-none-eabi-gdb -- -x scripts/pinetime.gdb board/target/thumbv7m-none-eabi/debug/pine_time.elf
