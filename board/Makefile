KERNEL_ADDRESS=0x00000
PLATFORM = pine_time
TARGET = thumbv7m-none-eabi
TOCKLOADER=tockloader
TOCKLOADER_JTAG_FLAGS = --jlink --board nrf52dk
TOCK_ARCH = cortex-m4

include ../tock/boards/Makefile.common

.PHONY: flash
flash: target/$(TARGET)/release/$(PLATFORM).bin
	$(TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) flash --address $(KERNEL_ADDRESS) $(TOCKLOADER_JTAG_FLAGS) $<
