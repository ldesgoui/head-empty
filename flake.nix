{
  description = "config library";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, rust-overlay, ... }:
    let
      supportedSystems = [
        "x86_64-linux"
      ];

      overlays = [
        rust-overlay.overlay
      ];

      genSystems = nixpkgs.lib.genAttrs supportedSystems;
      genSystemsWithPkgs = f: genSystems (system: f (import nixpkgs { inherit system overlays; }));
    in
    {
      devShell = genSystemsWithPkgs (pkgs: pkgs.mkShell {
        buildInputs = builtins.attrValues {
          inherit (pkgs)
            cargo-watch
            ;

          rust = pkgs.rust-bin.stable.latest.default;
        };

        RUST_BACKTRACE = "1";
      });
    };
}
