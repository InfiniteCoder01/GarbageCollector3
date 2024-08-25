{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = (import nixpkgs { inherit system; });
        in
        {
          devShell = with pkgs; mkShell {
            packages = [ trunk ];
            LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
              libGL
              xorg.libXrandr
              xorg.libXcursor
              xorg.libXi
              xorg.libX11
            ];
          };
          formatter = pkgs.nixpkgs-fmt;
        }
      );
}
