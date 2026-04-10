{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

{

  languages.rust.enable = true;

  packages = with pkgs; [
    git
    just
    surrealdb
    lld
    opentofu
    # Build dependencies for surrealdb-librocksdb-sys
    pkg-config
    openssl
    llvmPackages.libclang
    clang
    # Leptos dev tools (installed via cargo in enterShell for version compatibility)
    binaryen  # for wasm-opt
  ];

  # Environment variables for all processes
  env = {
    # Database configuration - use embedded mode by default
    LOAA_DB_MODE = "embedded";
    LOAA_DB_PATH = "./data/loaa.db";  # Relative to project root
    # Build dependencies for rocksdb/bindgen
    LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
  };

  # Note: cargo-leptos watch mode requires version matching - install manually if needed:
  #   cargo install cargo-leptos wasm-bindgen-cli
  # For now, you can run the server directly with:
  #   cargo build -p loaa-web --features ssr && ./target/debug/loaa-web

  dotenv.disableHint = true;

  # Process orchestration with process-compose
  # Run `devenv up` to start all services
  # Press Ctrl+C to stop all services
  processes = {
    # Web server with embedded database (no separate DB process needed in embedded mode)
    web = {
      exec = "cd crates/web && cargo leptos watch";
      process-compose = {
        availability = {
          restart = "on_failure";
          backoff_seconds = 2;
          max_restarts = 5;
        };
      };
    };
  };

  git-hooks.hooks.single-line-commit = {
    enable = true;
    name = "single-line commit";
    entry = "bash -c 'test $(grep -cv \"^#\" \"$1\") -le 1' --";
    stages = [ "commit-msg" ];
  };
}
