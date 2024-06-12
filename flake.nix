{
  description = "The cargo-changelog binary";
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustTarget = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustTarget;

        tomlInfo = craneLib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; };
        inherit (tomlInfo) pname version;
        src = ./.;

        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src;

          buildInputs = [ pkgs.pkg-config pkgs.openssl ];
        };

        changelog = craneLib.buildPackage {
          inherit cargoArtifacts src pname version;

          buildInputs = [ pkgs.pkg-config pkgs.openssl ];

          doCheck = false;
        };

      in
      rec {
        checks = {
          inherit changelog;

          changelog-clippy = craneLib.cargoClippy {
            inherit cargoArtifacts src;
            cargoClippyExtraArgs = "-- --deny warnings";
          };

          changelog-fmt = craneLib.cargoFmt {
            inherit src;
          };
        };

        apps.changelog = flake-utils.lib.mkApp {
          name = pname;
          drv = changelog;
        };
        apps.default = apps.changelog;

        packages.changelog = changelog;
        packages.default = packages.changelog;

        devShells.default = devShells.changelog;
        devShells.changelog = pkgs.mkShell {
          nativeBuildInputs = [
            rustTarget

            pkgs.openssl
            pkgs.pkg-config
          ];
        };
      }
    );
}
