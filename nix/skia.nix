# Prebuilt Skia derivation for freya-skia-bindings.

{
  lib,
  pkgs,
  rustPlatform,
  clangStdenv,
  fetchgit,
  linkFarm,
  runCommand,
  gn,
  ninja,
  python3,
  pkg-config,
  removeReferencesTo,
}:

let
  deps = import ./skia-deps.nix { inherit pkgs; };

  skia-source =
    let
      # chrome/m153 before fca11a08da, which changes GrDirectContext::flush from what freya-skia-safe 0.104 binds.
      repo = fetchgit {
        url = "https://skia.googlesource.com/skia";
        rev = "14d99416c52df27ad98e5ea2637121b867f54aad";
        hash = "sha256-wmtuC2scBQXONYNnpWpMqOYaCfb31904YP1MngxCM3w=";
      };
      externals = linkFarm "skia-externals" (
        lib.mapAttrsToList
          (name: value: { inherit name; path = fetchgit value; })
          (lib.importJSON ./skia-externals.json)
      );
    in
    runCommand "skia-source" { } ''
      cp -R ${repo} $out
      chmod -R +w $out
      ln -s ${externals} $out/third_party/externals
    '';
in
rustPlatform.buildRustPackage.override { stdenv = clangStdenv; } {
  pname = "skia-binaries";
  version = "0.104.0";

  src = ./skia-prebuild;
  cargoLock.lockFile = ./skia-prebuild/Cargo.lock;

  env = {
    SKIA_SOURCE_DIR = skia-source;
    SKIA_GN_COMMAND = "${gn}/bin/gn";
    SKIA_NINJA_COMMAND = "${ninja}/bin/ninja";
    FORCE_SKIA_BUILD = "1";
  };

  nativeBuildInputs = [
    pkg-config
    python3
    removeReferencesTo
  ];

  buildInputs = [
    rustPlatform.bindgenHook
  ]
  ++ deps;

  installPhase = ''
    runHook preInstall

    skia_out=$(find . -type d -path '*/release/build/freya-skia-bindings-*/out/skia' | head -n1)
    if [ -z "$skia_out" ]; then
      echo "skia-binaries: no encuentro el OUT_DIR/skia de freya-skia-bindings" >&2
      exit 1
    fi

    # Skia graba en las librerías la ruta de su árbol de fuentes; quítala para
    # no arrastrar todo Skia en el cierre.
    find "$skia_out" -type f -exec remove-references-to -t ${skia-source} {} +

    mkdir -p $out/skia-binaries
    cp "$skia_out"/*.a $out/skia-binaries/
    cp "$skia_out"/bindings.rs $out/skia-binaries/
    if [ -f "$skia_out/icudtl.dat" ]; then
      cp "$skia_out/icudtl.dat" $out/skia-binaries/
    fi
    # Mismo formato que publica rust-skia: skia-binaries-{key}.tar.gz con el
    # directorio skia-binaries dentro. La URL es local, así que el nombre no
    # necesita la clave; el contenido sí debe coincidir con las features.
    tar -czf $out/skia-binaries.tar.gz -C $out skia-binaries
    rm -rf $out/skia-binaries

    # Features con las que se compiló, para que el preBuild del paquete
    # compruebe que las de freya-engine/BarrierKit no cambiaron.
    cargo tree --offline -e features -i freya-skia-bindings --prefix none \
      | sed -n 's/^freya-skia-bindings feature "\([^"]*\)".*/\1/p' | sort -u > $out/features.txt

    runHook postInstall
  '';

  doCheck = false;

  disallowedReferences = [ skia-source ];

  meta = {
    description = "Prebuilt Skia for freya-skia-bindings 0.104.0";
    license = lib.licenses.bsd3;
    platforms = [ "x86_64-linux" "aarch64-linux" ];
  };
}
