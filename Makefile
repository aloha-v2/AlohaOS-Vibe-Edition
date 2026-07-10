# AlohaOS build + run helper.
#
# Requires:
# rustup target add x86_64-unknown-uefi x86_64-unknown-none
# qemu-system-x86_64
# an OVMF firmware image (pass its path via OVMF=...)

PROFILE ?= debug
KERNEL_FEATURES ?=
CARGO_FLAGS :=
KERNEL_FEATURE_FLAGS :=
ifeq ($(PROFILE),release)
CARGO_FLAGS += --release
endif
ifneq ($(strip $(KERNEL_FEATURES)),)
KERNEL_FEATURE_FLAGS += --features $(KERNEL_FEATURES)
endif

BOOT_EFI := target/x86_64-unknown-uefi/$(PROFILE)/alohaboot.efi
KERNEL_ELF := target/x86_64-unknown-none/$(PROFILE)/kernel
ESP := esp
OVMF ?= /usr/share/OVMF/OVMF_CODE.fd

.PHONY: all boot kernel esp run clean

all: esp

boot:
	cargo build -p alohaboot --target x86_64-unknown-uefi $(CARGO_FLAGS)

kernel:
	cargo build -p kernel --target x86_64-unknown-none $(CARGO_FLAGS) $(KERNEL_FEATURE_FLAGS)

esp: boot kernel
	mkdir -p $(ESP)/EFI/BOOT $(ESP)/alohaos
	cp $(BOOT_EFI) $(ESP)/EFI/BOOT/BOOTX64.EFI
	cp $(KERNEL_ELF) $(ESP)/alohaos/kernel.elf

run: esp
	qemu-system-x86_64 \
		-machine q35 \
		-m 256M \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF) \
		-drive format=raw,file=fat:rw:$(ESP)

clean:
	cargo clean
	rm -rf $(ESP)
