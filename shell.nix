let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs {};
in
pkgs.mkShell rec {
  packages = with pkgs; [
    # general
    rustup
    nixpkgs-fmt
    niv

    # integration test
    grub2  # for grub-file
    qemu
    xorriso

    (pkgs.writeShellScriptBin "run-integrationtest" ''
    ./integration-test/run.sh
    '')
  ];

  # To invoke "nix-shell" in the CI-runner, we need a global Nix channel.
  # For better reproducibility inside the Nix shell, we override this channel
  # with the pinned nixpkgs version.
  NIX_PATH = "nixpkgs=${sources.nixpkgs}";

  OVMF = "${pkgs.OVMF.fd}/FV/OVMF.fd";
}
