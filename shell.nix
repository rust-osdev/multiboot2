{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {
  packages = with pkgs; [
    # general
    rustup
    nixpkgs-fmt
    niv

    # integration test
    grub2
    qemu
    xorriso

    (pkgs.writeShellScriptBin "run-integrationtest" ''
    ./integration-test/run.sh
    '')
  ];
}
