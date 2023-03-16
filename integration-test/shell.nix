let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs {};
in
pkgs.mkShell rec {
  nativeBuildInputs = with pkgs; [
    grub2
    qemu
    rustup
    xorriso
  ];
}
