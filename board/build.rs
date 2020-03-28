fn main() {
    println!("cargo:rerun-if-changed=../tock/boards/kernel_layout.ld");
    println!("cargo:rerun-if-changed=../tock/boards/nordic/nrf52832_chip_layout.ld");
    println!("cargo:rerun-if-changed=layout.ld");
}
