{
    description = "Salon App Backend Development Shell";

    inputs = {
        nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    };

    outputs = {
        self,
        nixpkgs,
    }:
    let

      system = "x86_64-linux"; # change if needed
      pkgs = import nixpkgs { inherit system; };

    in
    {

      devShells.${system}.default = pkgs.mkShell {
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
            # Makes cargo work (Sets the proper version)
            RUSTUP_TOOLCHAIN=1.85.0
          '';

      };

    };

}
