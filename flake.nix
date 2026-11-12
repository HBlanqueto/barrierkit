{
  description = "BarrierKit: the application toolkit for Malaysia.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    { self, nixpkgs, crane }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      perSystem = nixpkgs.lib.genAttrs systems (system: let
        pkgs = import nixpkgs { inherit system; };
        lib = pkgs.lib;
        skia = pkgs.callPackage ./nix/skia.nix { };
        nativeDeps = with pkgs; [
          libglvnd
          mesa
          wayland
          libxkbcommon
          fontconfig
          freetype
          vulkan-loader
        ];
        videoDeps = with pkgs.gst_all_1; [
          gstreamer
          gst-plugins-base
          gst-plugins-good
          gst-plugins-bad
          gst-libav
          pkgs.libva
          pkgs.glib
        ];
        deps = nativeDeps ++ videoDeps;
        libraryPath = lib.makeLibraryPath deps;

        craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.symlinkJoin {
          name = "rust-toolchain";
          paths = [ p.cargo p.rustc p.clippy p.rustfmt ];
        });
        src = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            ./crates
            ./agents/recipes
            ./Cargo.toml
            ./Cargo.lock
          ];
        };
        commonArgs = {
          pname = "barrierkit";
          version = "0.1.0";
          inherit src;
          strictDeps = true;

          cargoLock = ./Cargo.lock;

          env = {
            BARRIERKIT_DEV_SHELL = "1";
            SKIA_BINARIES_URL = "file://${skia}/skia-binaries.tar.gz";
          };

          doCheck = false;

          CARGO_PROFILE = "dist";

          nativeBuildInputs = with pkgs; [ diffutils pkg-config ];
          buildInputs = nativeDeps;

          preBuild = ''
            version="$(cargo tree --offline -i freya-skia-bindings --depth 0 --prefix none \
              | sed -n 's/.*freya-skia-bindings v\([0-9.]*\).*/\1/p' | head -n1)"
            if [ "$version" != "0.104.0" ]; then
              echo "freya-skia-bindings $version no coincide con la derivación 0.104.0" >&2
              exit 1
            fi
            cargo tree --offline -e features -i freya-skia-bindings --prefix none \
              | sed -n 's/^freya-skia-bindings feature "\([^"]*\)".*/\1/p' | sort -u > skia-features.txt
            if ! diff -u ${skia}/features.txt skia-features.txt; then
              echo "las features de Skia cambiaron: actualiza nix/skia-prebuild" >&2
              exit 1
            fi
          '';
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        barrierkit = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });

        checkSrc = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [ ./crates ./agents/recipes ./scripts ./nix ./Cargo.toml ./Cargo.lock ];
        };

        fontsConf = pkgs.writeText "barrierkit-fonts.conf" ''
          <?xml version="1.0"?>
          <!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd">
          <fontconfig>
            <include ignore_missing="yes">/etc/fonts/fonts.conf</include>
            <dir>${pkgs.adwaita-fonts}/share/fonts</dir>
            <dir>${pkgs.geist-font}/share/fonts</dir>
            <dir>${pkgs.inter}/share/fonts</dir>
          </fontconfig>
        '';
      in {
        packages = {
          default = barrierkit;
          "barrierkit" = barrierkit;
          skia-binaries = skia;
        };

        checks = {
          design-tokens = pkgs.runCommand "check-design-tokens" {
            nativeBuildInputs = with pkgs; [ bash coreutils findutils gawk gnugrep ];
          } ''
            cd ${checkSrc}
            bash scripts/check-design-tokens.sh
            touch $out
          '';

          client-only-deps = pkgs.runCommand "check-client-only-deps" {
            nativeBuildInputs = with pkgs; [ cargo rustc jq ];
          } ''
            export CARGO_HOME=$TMPDIR
            ln -s ${craneLib.vendorCargoDeps { inherit src; }}/config.toml $CARGO_HOME/config.toml
            cd ${checkSrc}
            bash scripts/check-client-only-deps.sh
            touch $out
          '';
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            clippy
            rustfmt
            pkg-config
            jq
            mold
            shellcheck
            tracy
            ffmpeg
            yt-dlp
          ];

          buildInputs = deps ++ [
            pkgs.adwaita-fonts
            pkgs.geist-font
            pkgs.inter
          ];

          RUSTFLAGS = "-C link-arg=-fuse-ld=mold -C link-arg=-Wl,-rpath,${libraryPath}";
          FONTCONFIG_FILE = fontsConf;
          BARRIERKIT_DEV_SHELL = "1";
          SKIA_BINARIES_URL = "file://${skia}/skia-binaries.tar.gz";

          shellHook = ''
            export LD_LIBRARY_PATH="${libraryPath}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
            mesa_icds=$(printf '%s:' ${pkgs.mesa}/share/vulkan/icd.d/*.json)
            export VK_ADD_DRIVER_FILES="''${mesa_icds%:}''${VK_ADD_DRIVER_FILES:+:$VK_ADD_DRIVER_FILES}"
            unset mesa_icds
            export GST_PLUGIN_SYSTEM_PATH_1_0="${lib.makeSearchPathOutput "lib" "lib/gstreamer-1.0" videoDeps}"
            # souphttpsrc needs a GIO TLS backend to open https:// sources.
            export GIO_EXTRA_MODULES="${pkgs.glib-networking}/lib/gio/modules"
            export LIBVA_DRIVERS_PATH="${pkgs.mesa}/lib/dri"
          '';
        };
      });
    in
    {
      packages = nixpkgs.lib.mapAttrs (_: s: s.packages) perSystem;
      checks = nixpkgs.lib.mapAttrs (_: s: s.checks) perSystem;
      devShells = nixpkgs.lib.mapAttrs (_: s: s.devShells) perSystem;
    };
}
