{
  description = "multiboot2-rs";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    inputs@{ self, nixpkgs, ... }:
    let
      # Systems definition for dev shells and exported packages,
      # independent of the NixOS configurations and modules defined here. We
      # just use "every system" here to not restrict any user. However, it
      # likely happens that certain packages don't build for/under certain
      # systems.
      systems = nixpkgs.lib.systems.flakeExposed;
      forAllSystems =
        function: nixpkgs.lib.genAttrs systems (system: function nixpkgs.legacyPackages.${system});
    in
    {
      formatter = forAllSystems (pkgs: pkgs.nixfmt-tree);
      devShells = forAllSystems (pkgs: {
        default = pkgs.callPackage ./shell.nix { };
      });
    };
}
