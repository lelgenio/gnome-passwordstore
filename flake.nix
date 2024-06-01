{
  description = "Build a cargo project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-24.05";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = inputs@{ self, nixpkgs, crane, flake-utils, advisory-db, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (import inputs.rust-overlay)
          ];

        };

        inherit (pkgs) lib;

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = craneLib.cleanCargoSource ./.;

        buildInputs = with pkgs; [
          pkg-config
          gtk4
          libadwaita
        ];

        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src buildInputs;
        };

        crate = craneLib.buildPackage {
          inherit cargoArtifacts src buildInputs;
        };
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit crate;

          crate-clippy = craneLib.cargoClippy {
            inherit cargoArtifacts src buildInputs;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          };

          crate-doc = craneLib.cargoDoc {
            inherit cargoArtifacts src buildInputs;
          };

          crate-fmt = craneLib.cargoFmt {
            inherit src;
          };

          crate-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };

          crate-nextest = craneLib.cargoNextest {
            inherit cargoArtifacts src buildInputs;
            partitions = 1;
            partitionType = "count";
          };
        } // lib.optionalAttrs (system == "x86_64-linux") {
          crate-coverage = craneLib.cargoTarpaulin {
            inherit cargoArtifacts src;
          };
        };

        packages.default = crate;

        apps.default = flake-utils.lib.mkApp {
          drv = crate;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks;
          inherit buildInputs;
          nativeBuildInputs = with pkgs; [
            rustToolchain
            cargo-watch
            cargo-feature
          ];
        };
      });
}
