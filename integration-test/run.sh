#!/usr/bin/env bash

DIR=$(dirname "$(realpath "$0")")
cd "$DIR" || exit

# Helper script that runs the actual script in a Nix shell.
nix-shell --run ./.run.sh
