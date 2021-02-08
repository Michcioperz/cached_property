with import <nixpkgs> {};
mkShell {
  buildInputs = [ stdenv rustc cargo rustfmt ];
}
