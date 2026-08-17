{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell rec {
  packages = with pkgs; [
    # general
    rustup

    # integration test
    grub2  # for grub-file
    qemu
    xorriso

    (pkgs.writeShellScriptBin "run-integrationtest" ''
      ./integration-test/run.sh
    '')
  ];

  OVMF = "${pkgs.OVMF.fd}/FV/OVMF.fd";
}
