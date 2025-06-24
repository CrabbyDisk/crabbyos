fn main() {
    println!("cargo::rerun-if-changed=linker.ld");
    println!("cargo::rerun-if-changed=src/entry.s");
    println!("cargo:rustc-link-arg=-Tlinker.ld");
}
