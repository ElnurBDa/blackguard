{
  description = "Blackguard — a beautiful terminal implementation of the card game Scoundrel";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      pkgsFor = system: import nixpkgs { inherit system; };
      forAll = f: nixpkgs.lib.genAttrs systems (system: f (pkgsFor system) system);
    in
    {
      packages = forAll (
        pkgs: _system: {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "blackguard";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            meta = with pkgs.lib; {
              description = "Blackguard: a beautiful terminal implementation of the card game Scoundrel";
              homepage = "https://github.com/ElnurBDa/blackguard";
              license = licenses.mit;
              mainProgram = "blackguard";
            };
          };
        }
      );

      devShells = forAll (
        pkgs: _system: {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustc
              cargo
              clippy
              rustfmt
              rust-analyzer
            ];
          };
        }
      );

      apps = forAll (
        _pkgs: system: {
          default = {
            type = "app";
            program = "${self.packages.${system}.default}/bin/blackguard";
          };
        }
      );

      formatter = forAll (pkgs: _system: pkgs.nixfmt-rfc-style);
    };
}
