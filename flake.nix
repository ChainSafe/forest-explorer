{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    utils.url = "github:numtide/flake-utils";
    worker-build.url = "github:lemmih/nix-flakes?dir=worker-build";
    worker-build.inputs.nixpkgs.follows = "nixpkgs";
    wrangler.url = "github:ryand56/wrangler";
    wrangler.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    alejandra.url = "github:kamadorueda/alejandra/3.1.0";
    alejandra.inputs.nixpkgs.follows = "nixpkgs";
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    worker-build,
    wrangler,
    rust-overlay,
    alejandra,
    crane,
  }:
    utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };
        # wasm-pack usually downloads the right version of wasm-bindgen-cli, but
        # nix doesn't allow network access while building. We get around this by
        # manually installing the right version.
        pinned-wasm-bindgen-cli = pkgs.wasm-bindgen-cli.override {
          version = "0.2.100";
          hash = "sha256-3RJzK7mkYFrs7C/WkhW9Rr4LdP5ofb2FdYGz1P7Uxog=";
          cargoHash = "sha256-tD0OY2PounRqsRiFh8Js5nyknQ809ZcHMvCOLrvYHRE=";
        };
        worker-build-bin = worker-build.packages.${system}.default;
        wrangler-bin = wrangler.packages.${system}.default;

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        # Initialize crane with our custom toolchain
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Custom source filter that extends crane's default cargo filtering
        # but also excludes wrangler.toml
        customFilter = path: type:
          (craneLib.filterCargoSources path type) && baseNameOf path != "wrangler.toml";

        # Common source definition for all derivations
        commonSrc = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = customFilter;
        };

        explorer-client-deps = craneLib.buildDepsOnly {
          src = commonSrc;
          cargoExtraArgs = "--target wasm32-unknown-unknown --features hydrate --no-default-features";
          doCheck = false;
        };

        # Create a derivation for building the client-side Wasm using crane
        explorer-client = craneLib.buildPackage {
          src = commonSrc;
          cargoArtifacts = explorer-client-deps;
          buildPhaseCargoCommand = ''
            HOME=$PWD/tmp wasm-pack build \
              --out-dir pkg \
              --mode no-install \
              --no-typescript \
              --release \
              --target web \
              --out-name client \
              --features hydrate \
              --no-default-features
          '';
          doNotPostBuildInstallCargoBinaries = true;
          installPhaseCommand = ''
            mkdir -p $out/pkg
            cp -r pkg/* $out/pkg/
          '';
          doCheck = false;

          nativeBuildInputs = with pkgs; [
            wasm-pack
            pinned-wasm-bindgen-cli
            binaryen
          ];
        };

        explorer-server-deps = craneLib.buildDepsOnly {
          src = commonSrc;
          cargoExtraArgs = "--target wasm32-unknown-unknown --features ssr --no-default-features";
          doCheck = false;
        };

        # Create a derivation for building the server-side Wasm using crane
        explorer-server = craneLib.buildPackage {
          src = commonSrc;
          cargoArtifacts = explorer-server-deps;
          buildPhaseCargoCommand = "HOME=$PWD/tmp worker-build --release --features ssr --no-default-features";
          doNotPostBuildInstallCargoBinaries = true;
          doCheck = false;
          installPhaseCommand = ''
            mkdir -p $out/build
            cp -r build/* $out/build/
          '';

          nativeBuildInputs = with pkgs; [
            worker-build-bin
            pinned-wasm-bindgen-cli
            binaryen
            esbuild
          ];
        };

        # Create a derivation for the styles
        explorer-styles = pkgs.stdenv.mkDerivation {
          name = "explorer-styles";
          src = commonSrc;

          nativeBuildInputs = with pkgs; [
            tailwindcss
          ];

          buildPhase = ''
            tailwindcss --minify -i ${./style/tailwind.css} -o style.css
          '';

          installPhase = ''
            mkdir -p $out
            cp style.css $out/style.css
          '';
        };

        # Create the main explorer derivation that combines everything
        explorer = pkgs.symlinkJoin {
          name = "explorer";
          paths = [
            (pkgs.runCommand "explorer-static" {} ''
              mkdir -p $out/assets
              cp -r ${./public}/* $out/assets/
              cp ${explorer-styles}/style.css $out/assets/style.css
              cp -r ${explorer-client}/pkg $out/assets/
            '')
            explorer-server
          ];
        };

        # Create a development environment with a script to run wrangler
        explorer-preview = pkgs.writeScriptBin "explorer-preview" ''
          #!${pkgs.bash}/bin/bash

          # Create a temporary directory for the development environment
          WORK_DIR=$(mktemp -d)

          # Link the necessary directories
          ln -s ${explorer}/assets $WORK_DIR/assets
          ln -s ${explorer}/build $WORK_DIR/build

          # Link the wrangler configuration
          ln -s ${./wrangler.toml} $WORK_DIR/wrangler.toml

          # Change to the work directory
          cd $WORK_DIR

          # Run wrangler in development mode
          exec ${wrangler-bin}/bin/wrangler dev --env prebuilt --live-reload false
        '';

        # Create a deployment script
        explorer-deploy = pkgs.writeScriptBin "explorer-deploy" ''
          #!${pkgs.bash}/bin/bash

          # Create a temporary directory for the deployment
          WORK_DIR=$(mktemp -d)
          trap 'rm -rf "$WORK_DIR"' EXIT

          # Link the necessary directories
          ln -s ${explorer}/assets $WORK_DIR/assets
          ln -s ${explorer}/build $WORK_DIR/build

          # Link the wrangler configuration
          ln -s ${./wrangler.toml} $WORK_DIR/wrangler.toml

          # Change to the work directory
          cd $WORK_DIR

          # Run wrangler deploy with any additional arguments
          exec ${wrangler-bin}/bin/wrangler deploy --env prebuilt "$@"
        '';
      in {
        packages = {
          inherit explorer explorer-client explorer-server explorer-styles;
          wrangler = wrangler-bin;
          default = explorer;
        };

        # Add the development app
        apps.default = {
          type = "app";
          program = "${explorer-preview}/bin/explorer-preview";
        };

        # Create a deployment script
        apps.deploy = {
          type = "app";
          program = "${explorer-deploy}/bin/explorer-deploy";
        };

        # Development shell with required tools
        devShell = craneLib.devShell {
          buildInputs = with pkgs; [
            pinned-wasm-bindgen-cli
            wasm-pack
            worker-build-bin
            wrangler-bin
            binaryen
            corepack
            cargo-deny
            cargo-spellcheck
            taplo
          ];

          shellHook = ''
            echo "🌲 Welcome to Forest Explorer Development Shell 🌲"
            echo ""
            echo "Installed tools: yarn, wrangler, wasm-opt, wasm-pack, worker-build, wasm-bindgen"
            echo "Installed lints: cargo-deny, cargo-spellcheck, taplo"
            echo ""
            echo "To start the development server:"
            echo "  yarn dev"
            echo ""
            echo "To validate your changes:"
            echo "  make lint-all"
            echo ""
          '';
        };

        formatter = alejandra.packages.${system}.default;
      }
    );
}
