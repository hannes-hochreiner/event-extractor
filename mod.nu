export def test [] {
  ^cargo test
}

export def build [] {
  test
  ^cargo build
}

export def update_flake_lock [] {
  ^podman "run" "--rm" "-it" "-v" $"($env.PWD):/workspace:z" "nixos/nix" "bash" "-c" "nix flake update --extra-experimental-features nix-command --extra-experimental-features flakes /workspace"
}
