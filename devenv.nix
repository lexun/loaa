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
    # SurrealDB for embedded database (RocksDB backend)
    # Note: SurrealDB is a Rust crate, but we need RocksDB system libraries
    rocksdb
  ];
}
