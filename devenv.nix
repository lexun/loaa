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
  ];

  # Process orchestration with process-compose
  # Run `devenv up` to start all services
  # Press Ctrl+C to stop all services
  processes = {
    # SurrealDB database server (memory mode for development, easily swappable for production)
    db = {
      exec = "surreal start --log info --user root --pass root --bind 127.0.0.1:8000 memory";
      process-compose = {
        availability = {
          restart = "on_failure";
          backoff_seconds = 2;
          max_restarts = 5;
        };
        readiness_probe = {
          exec.command = "curl -f http://127.0.0.1:8000/health || exit 1";
          initial_delay_seconds = 2;
          period_seconds = 2;
          timeout_seconds = 1;
          success_threshold = 1;
          failure_threshold = 3;
        };
      };
    };

    # Web server (depends on database being healthy)
    web = {
      exec = "cd crates/web && cargo leptos watch";
      process-compose = {
        depends_on.db.condition = "process_healthy";
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
