{ pkgs, lib, config, inputs, ... }:

let
  libPath = builtins.replaceStrings [" "] [":"] (lib.makeLibraryPath [
    pkgs.libxkbcommon
    pkgs.libGL
    pkgs.wayland
    pkgs.xorg.libX11
    pkgs.xorg.libXrandr
    pkgs.xorg.libXi
    pkgs.xorg.libXcursor
    pkgs.pkg-config
  ]);
in {
  env.WINIT_UNIX_BACKEND = "wayland";
  env.LD_LIBRARY_PATH = libPath;

  # https://devenv.sh/packages/
  packages = [
    pkgs.clang
    pkgs.mold
  ];

  # https://devenv.sh/languages/
  languages.rust = {
    enable = true;
    channel = "nightly";
    components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" "rust-src" "llvm-tools-preview" "rustc-codegen-cranelift-preview" ];
    rustflags = "-Clink-arg=-Wl,-rpath,${libPath} -Clink-arg=-fuse-ld=${pkgs.mold}/bin/mold";
  };

  # https://devenv.sh/processes/
  # processes.dev.exec = "${lib.getExe pkgs.watchexec} -n -- ls -la";

  # https://devenv.sh/services/
  # services.postgres.enable = true;

  # https://devenv.sh/scripts/
   scripts.hello.exec = ''
    echo ${libPath}
   '';

  # https://devenv.sh/basics/
#  enterShell = ''
#     hello         # Run scripts directly
#    git --version # Use packages
#  '';

  # https://devenv.sh/tasks/
  # tasks = {
  #   "myproj:setup".exec = "mytool build";
  #   "devenv:enterShell".after = [ "myproj:setup" ];
  # };

  # https://devenv.sh/tests/
#  enterTest = ''
#    echo "Running tests"
#    git --version | grep --color=auto "${pkgs.git.version}"
#  '';

  # https://devenv.sh/git-hooks/
  # git-hooks.hooks.shellcheck.enable = true;

  # See full reference at https://devenv.sh/reference/options/
}
