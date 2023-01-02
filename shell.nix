with import <nixpkgs> {};
pkgsCross.riscv64-embedded.mkShell {
    nativeBuildInputs = [ file rustup minicom stm32flash ];
}
