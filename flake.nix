{
  description = "sbi development flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }: 
    flake-utils.lib.eachDefaultSystem(system: 
      let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust-bin = pkgs.rust-bin.selectLatestNightlyWith(toolchain: toolchain.default.override {
          extensions = ["rust-src"];
          targets = ["riscv64imac-unknown-none-elf" "riscv32imac-unknown-none-elf"];
        });
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.nil
            rust-bin
          ];
        };
      }
    );
}