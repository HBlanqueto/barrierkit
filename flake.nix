{
  description = "BarrierKit: the application toolkit for MalaysiaDreams.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    { self, nixpkgs, crane }:
    let
      systems = [ "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;

      nativeDeps = pkgs: with pkgs; [
        libglvnd
        libdrm
        libgbm
        mesa
        wayland
        libxkbcommon
        fontconfig
        freetype
      ];
    in
    {
      packages = forAllSystems (system: let
        pkgs = import nixpkgs { inherit system; };
        lib = pkgs.lib;
        craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.symlinkJoin {
          name = "rust-toolchain";
          paths = [ p.cargo p.rustc p.clippy p.rustfmt ];
        });
        src = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            ./crates
            ./protocols
            ./Cargo.toml
            ./Cargo.lock
          ];
        };
        skia = pkgs.callPackage ./nix/skia.nix { };
        hasFreyaSkia = lib.hasInfix "freya-skia-bindings" (builtins.readFile ./Cargo.lock);
        commonArgs = {
          pname = "barrierkit";
          version = "0.1.0";
          inherit src;
          strictDeps = true;

          cargoLock = ./Cargo.lock;

          env = {
            MALAYSIA_DEV_SHELL = "1";
          }
          // lib.optionalAttrs hasFreyaSkia {
            SKIA_BINARIES_URL = "file://${skia}/skia-binaries.tar.gz";
          };

          doCheck = false;

          CARGO_PROFILE = "dist";

          nativeBuildInputs = with pkgs; [ diffutils pkg-config ];
          buildInputs = nativeDeps pkgs;

          preBuild = ''
            if cargo tree --offline -i freya-skia-bindings --depth 0 --prefix none >/dev/null 2>&1; then
              version="$(cargo tree --offline -i freya-skia-bindings --depth 0 --prefix none \
                | sed -n 's/.*freya-skia-bindings v\([0-9.]*\).*/\1/p' | head -n1)"
              if [ "$version" != "0.98.0" ]; then
                echo "freya-skia-bindings $version no coincide con la derivación 0.98.0" >&2
                exit 1
              fi
              cargo tree --offline -e features -i freya-skia-bindings --prefix none \
                | sed -n 's/^freya-skia-bindings feature "\([^"]*\)".*/\1/p' | sort -u > skia-features.txt
              if ! diff -u ${skia}/features.txt skia-features.txt; then
                echo "las features de Skia cambiaron: actualiza nix/skia-prebuild" >&2
                exit 1
              fi
            else
              echo "freya-skia-bindings no está en el grafo todavía; Skia precompilado queda listo" >&2
            fi
          '';
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        barrierkit = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });
      in {
        default = barrierkit;
        "barrierkit" = barrierkit;
        skia-binaries = skia;
      });

      checks = forAllSystems (system: let
        pkgs = import nixpkgs { inherit system; };
        lib = pkgs.lib;
        checkSrc = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [ ./crates ./scripts ./nix ./protocols ./Cargo.toml ./Cargo.lock ];
        };
      in {
        tengen-generated = let
          tengen = import ./crates/vow-design/tengen;
        in pkgs.runCommand "check-tengen-generated" {
          nativeBuildInputs = with pkgs; [ diffutils ];
        } ''
          cd ${checkSrc}
          diff -u crates/vow-design/tengen/generated/tengen.slint \
            ${pkgs.writeText "tengen.slint" tengen.slint}
          diff -u crates/vow-design/tengen/generated/tengen.css \
            ${pkgs.writeText "tengen.css" tengen.css}
          diff -u crates/vow-design/tengen/generated/Tengen.qml \
            ${pkgs.writeText "Tengen.qml" tengen.qml}
          touch $out
        '';

        design-tokens = pkgs.runCommand "check-design-tokens" {
          nativeBuildInputs = with pkgs; [ bash coreutils findutils gawk gnugrep ];
        } ''
          cd ${checkSrc}
          bash scripts/check-design-tokens.sh
          touch $out
        '';

        client-only-deps = pkgs.runCommand "check-client-only-deps" {
          nativeBuildInputs = with pkgs; [ cargo rustc ];
        } ''
          cd ${checkSrc}
          bash scripts/check-client-only-deps.sh
          touch $out
        '';
      });

      devShells = forAllSystems (system: let
        pkgs = import nixpkgs { inherit system; };
        deps = nativeDeps pkgs;
        skia = pkgs.callPackage ./nix/skia.nix { };
        hasFreyaSkia = pkgs.lib.hasInfix "freya-skia-bindings" (builtins.readFile ./Cargo.lock);
        fontsConf = pkgs.writeText "barrierkit-fonts.conf" ''
          <?xml version="1.0"?>
          <!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd">
          <fontconfig>
            <include ignore_missing="yes">/etc/fonts/fonts.conf</include>
            <dir>${pkgs.geist-font}/share/fonts</dir>
            <dir>${pkgs.inter}/share/fonts</dir>
          </fontconfig>
        '';
      in {
        default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            clippy
            rustfmt
            pkg-config
            mold
            shellcheck
          ];

          buildInputs = deps ++ [
            pkgs.geist-font
            pkgs.inter
          ];

          RUSTFLAGS = "-C link-arg=-fuse-ld=mold -C link-arg=-Wl,-rpath,${pkgs.lib.makeLibraryPath deps}";
          FONTCONFIG_FILE = fontsConf;
          MALAYSIA_DEV_SHELL = "1";

          shellHook = ''
            ${pkgs.lib.optionalString hasFreyaSkia ''
              export SKIA_BINARIES_URL="file://${skia}/skia-binaries.tar.gz"
            ''}
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath deps}$LD_LIBRARY_PATH"
            export __EGL_VENDOR_LIBRARY_DIRS="${pkgs.mesa}/share/glvnd/egl_vendor.d''${__EGL_VENDOR_LIBRARY_DIRS:+:$__EGL_VENDOR_LIBRARY_DIRS}"
            export LIBGL_DRIVERS_PATH="${pkgs.mesa}/lib/dri''${LIBGL_DRIVERS_PATH:+:$LIBGL_DRIVERS_PATH}"
            export GBM_BACKENDS_PATH="${pkgs.mesa}/lib/gbm''${GBM_BACKENDS_PATH:+:$GBM_BACKENDS_PATH}"
          '';
        };
      });
    };
}
