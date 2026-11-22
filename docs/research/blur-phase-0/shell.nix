let
  flake = builtins.getFlake "git+file:///work";
  system = builtins.currentSystem;
  pkgs = import flake.inputs.nixpkgs { inherit system; };
  base = flake.devShells.${system}.default;
in
base.overrideAttrs (old: {
  buildInputs = old.buildInputs ++ (with pkgs; [ libinput libdrm libgbm seatd pam udev libheif libva ]);
  nativeBuildInputs = old.nativeBuildInputs ++ [ pkgs.rustPlatform.bindgenHook ];
})
