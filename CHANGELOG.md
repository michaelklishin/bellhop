# Change Log

## 1.4.0 (in development)

### Enhancements

 * (none yet)


## 1.3.0 (Feb 8, 2026)

### Enhancements

 * Introduce a new set of repositories, `cli-tools` (for `rabbitmqadmin`, `rabbitmq-lqt` and such)
 * `rabbitmq deb import-from-github` and `cli-tools deb import-from-github`: import `.deb` packages directly from GitHub releases
 * A new command, `watch`: a directory watcher that monitors subdirectories for `.deb` files and automatically imports them
   into the right set of `aptly` repositories managed by Bellhop
 * `repositories set-up` is a new command that creates all expected aptly repositories (idempotently)


## 1.2.0 (Dec 16, 2025)

### Enhancements

 * `rabbitmq deb remove` and `erlang deb remove` now support archive arguments. All packages discovered
   in the archive will be deleted from the specified repositories
 * Improved logging

### Bug Fixes

 * File extension detection is now case-insensitive
