fn main() {
    println!("cargo:rerun-if-changed=../tock/boards/kernel_layout.ld");
    println!("cargo:rerun-if-changed=layout.ld");
}
