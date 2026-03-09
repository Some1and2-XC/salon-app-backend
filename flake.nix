{
    description = "Salon App Backend Development Shell";

    inputs = {
        nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
        rust-overlay.url = "github:oxalica/rust-overlay";
    };

    outputs = {
        self,
        nixpkgs,
        rust-overlay,
    }:
    let
      system = "x86_64-linux"; # change if needed
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };

    in
    {

      devShells.${system} = {
          default = pkgs.mkShell {

              buildInputs = [
                # Rust deps
                pkgs.cargo
                pkgs.rustc
                pkgs.trunk # Web server runtime
                # C Packages deps
                pkgs.cmake
                pkgs.ninja
                pkgs.pkg-config
                # Shared Libraries
                pkgs.openssl
              ];

              shellHook = ''
                # Set the status
                export PS1="\n\[\033[1;31m\](Salon Back-End) \[\033[1;34m\]\w\[\033[0m\] \n$ "
                # Make colors work
                export TERM="xterm"
                # Disables potential analytics from swagger-ui
                export SCARF_ANALYTICS=false
                # Makes cargo work (Sets the proper version)
                RUSTUP_TOOLCHAIN=1.85.0
              '';

          };

          cross = pkgs.mkShell {
              buildInputs = [

                (pkgs.rust-bin.stable.latest.default.override {
                    targets = [ "x86_64-unknown-linux-musl" ];
                })

                # C Packages deps
                pkgs.cmake
                pkgs.ninja
                pkgs.pkg-config

                pkgs.pkgsCross.musl64.stdenv.cc
                pkgs.pkgsCross.musl64.openssl
                pkgs.pkgsCross.musl64.sqlite

              ];

              shellHook = ''

                # Sets open SSL things for cross.
                export OPENSSL_DIR="${pkgs.pkgsCross.musl64.openssl.dev}"
                export OPENSSL_LIB_DIR="${pkgs.pkgsCross.musl64.openssl.out}/lib"
                export OPENSSL_INCLUDE_DIR="${pkgs.pkgsCross.musl64.openssl.dev}/include"
                # Sets C compiler to be cross version
                export CC_x86_64_unknown_linux_musl="${pkgs.pkgsCross.musl64.stdenv.cc}/bin/x86_64-unknown-linux-musl-cc"

                # Set the status
                export PS1="\n\[\033[1;31m\](Salon Back-End --CROSS--) \[\033[1;34m\]\w\[\033[0m\] \n$ "
                # Make colors work
                export TERM="xterm"
                # Disables potential analytics from swagger-ui
                export SCARF_ANALYTICS=false
                # Makes cargo work (Sets the proper version)
                RUSTUP_TOOLCHAIN=1.85.0
              '';

          };

      };



    };

}
