{
  description = "A devShell example";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgsCross = nixpkgs.legacyPackages.x86_64-linux.pkgsCross.aarch64-multiplatform;
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in {
        devShells.default = with pkgs;
          mkShell {
            nativeBuildInputs = [
            ];
            buildInputs = [
              openssl
              pkg-config
              eza
              fd
              ripgrep
              (rust-bin.selectLatestNightlyWith (toolchain:
                toolchain.default.override {
                  extensions = ["rust-src"];
                  targets = ["aarch64-unknown-linux-gnu"];
                }))
              rust-analyzer
              protobuf
              scc
              cargo-audit
              cargo-deb
              docker-compose-language-service
              dockerfile-language-server-nodejs
              yaml-language-server
            ];
            env = {
              CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "${pkgsCross.gcc}";
            };
          };
      }
    );
}
