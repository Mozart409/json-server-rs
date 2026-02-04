{ 
  description = "json-server-rs development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        # Use specific Rust version
        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rust
            bacon
            lefthook
            just
            cargo-watch
            cargo-audit
            # Additional useful tools
            clang
            pkg-config
          ];
          
          shellHook = ''
            echo "json-server-rs dev shell"
            echo "Rust: $(rustc --version)"
            echo "Available tools: bacon, lefthook, just, cargo-watch, cargo-audit"
          '';
        };
      }
    );
}