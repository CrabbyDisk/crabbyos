{
  pkgs ?
    import <nixpkgs> {
      overlays = [(import "${fetchTarball "https://github.com/nix-community/fenix/archive/monthly.tar.gz"}/overlay.nix")];
      config.allowUnfree = true;
    },
}:
pkgs.mkShell.override {
  stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.clangStdenv;
} rec {
  # Get dependencies from the main package
  # Additional tooling
  buildInputs = with pkgs; [
    (fenix.complete.toolchain)
    fenix.rust-analyzer
    bacon
    qemu
    xorriso
    just
    cargo-binutils
    gdb
  ];

  RUST_SRC_PATH = "${
    (pkgs.fenix.complete.toolchain)
  }/lib/rustlib/src/rust/library";
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
  OVMF_PATH = "${pkgs.OVMF.fd}/FV";
}
