# Package managers
Topiary has been packaged for some package managers.

## Nix(pkgs)
To install Topiary from nix use whichever way you are familiar with. For instance:

### `configuration.nix`
```nix
environment.systemPackages = with pkgs; [
  topiary
];
```

### `home.nix`
```nix
home.packages = with pkgs; [
  topiary
];
```

### nix install
#### On NixOS
```bash
# without flakes:
nix-env -iA nixos.topiary
# with flakes:
nix profile install nixpkgs#topiary
```
#### On Non NixOS
```bash
# without flakes:
nix-env -iA nixpkgs.topiary
# with flakes:
nix profile install nixpkgs#topiary
```

### `nix-shell`
To temporarily add `topiary` to your path, use:
```bash
# without flakes:
nix-shell -p topiary
# with flakes:
nix shell nixpkgs#topiary
```

## Arch Linux (AUR)
```bash
yay -S topiary
```

## Cargo
```bash
cargo install -p topiary-cli
```
