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

  # To invoke "nix-shell" in the CI-runner, we need a global Nix channel.
  # For better reproducibility inside the Nix shell, we override this channel
  # with the pinned nixpkgs version.
  NIX_PATH = "nixpkgs=${sources.nixpkgs}";
}
