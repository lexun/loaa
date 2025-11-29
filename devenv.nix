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
  ];

  # Environment variables for all processes
  env = {
    # Database configuration - use embedded mode by default
    LOAA_DB_MODE = "embedded";
    LOAA_DB_PATH = "./data/loaa.db";  # Relative to project root
  };

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

  # Enable process-compose with TUI
  process.managers.process-compose.enable = true;
}
