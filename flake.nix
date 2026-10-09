{
  description = "MediOSS dev shell: Rust + bun + ffmpeg + Playwright's Chromium runtime libraries";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system);
    in
    {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
          # Chromium's runtime library set for Playwright's downloaded headless shell.
          chromiumLibs = with pkgs; [
            stdenv.cc.cc.lib
            nss
            nspr
            at-spi2-atk
            at-spi2-core
            dbus
            glib
            gtk3
            alsa-lib
            libdrm
            libxkbcommon
            libX11
            libxcb
            libXcomposite
            libXdamage
            libXext
            libXfixes
            libXrandr
            mesa
            libgbm
            expat
            systemdMinimal
            udev
          ];
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              # Rust
              cargo
              rustc
              rustfmt
              clippy
              cargo-watch
              # Frontend
              bun
              # Media pipeline
              ffmpeg
              # Tooling
              openssl
            ];
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath chromiumLibs;
            shellHook = ''
              echo "MediOSS dev shell: cargo/bun/ffmpeg + Chromium libs on LD_LIBRARY_PATH"
            '';
          };
        });
    };
}
