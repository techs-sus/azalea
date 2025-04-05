{
  description = "a flake which contains a devshell, package, and formatter";

  # binary cache for wally-nix (needs to be in the root flake too)
  nixConfig = {
    extra-substituters = "https://techs-sus-wally-nix.cachix.org";
    extra-trusted-public-keys = "techs-sus-wally-nix.cachix.org-1:hOye+Fj1heELMgDJOzDoQnFLsQA/kVN0ZVRnZmsAyB4=";
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };

    wally-nix.url = "github:techs-sus/wally-nix";

    run-in-cloud = {
      url = "github:techs-sus/run-in-cloud";
      inputs = {
        flake-utils.follows = "flake-utils";
        rust-overlay.follows = "rust-overlay";
      };
    };
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    wally-nix,
    run-in-cloud,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        wally = wally-nix.packages.${system}.default;
        run-in-cloud-pkg = run-in-cloud.packages.${system}.default;
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            (pkgs.rust-bin.stable.latest.default.override {
              extensions = [
                "rust-analyzer"
                "rust-src"
              ];
            })
          ];

          packages = [
            pkgs.bun
            pkgs.rojo
            pkgs.hyperfine

            wally
            run-in-cloud-pkg # a run-in-roblox replacement
          ];

          shellHook = "";
        };

        formatter = pkgs.nixfmt-tree;
        packages.default = pkgs.callPackage ./. {
          inherit inputs;
          inherit pkgs;
        };
      }
    );
}
