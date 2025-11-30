# Bellhop, a Package Repository Helper

This is a tool that helps maintaining a set of Debian (apt) repositories. It fills in the gaps
in `aptly` functionality that are essential for some very specific use cases:

 * When a project supports N distributions (both Debian and Ubuntu) for each package
 * When a single tool can be packaged as M packages (e.g. Erlang/OTP on Debian is 50-60 packages depending on the series)


## Usage

Use `help` to export the available commands.

```shell
# Add RabbitMQ 4.2.1 to Debian Bookworm and Ubuntu Noble
bellhop rabbitmq deb add -p rabbitmq-server_4.2.1-1_all.deb -d bookworm,noble

# Add Erlang packages from an archive to all supported distributions
bellhop erlang deb add -p erlang-27.3.4.5.tar.gz --all
bellhop erlang deb add -p erlang-27.3.4.6.zip --all

# Create a snapshot before publishing
bellhop rabbitmq snapshot take -d bookworm --suffix 30-Nov-24
bellhop rabbitmq deb publish -d bookworm

# Remove a version in the Bookworm repo
bellhop rabbitmq deb remove -v 4.1.5-1 -d bookworm

# Add a new version to the Bookworm repo, publish that repository's snapshot
bellhop rabbitmq deb add -p rabbitmq-server_4.1.6-1_all.deb -d bookworm
bellhop rabbitmq deb publish -d bookworm
```

## Contributing

Contributions to this project are very welcome. Please refer to the [CONTRIBUTING.md](CONTRIBUTING.md) to learn more.


## License

This software is dual-licensed under the MIT License and the Apache License, Version 2.0.


## Copyright

(c) 2025-2026 Michael S. Klishin and Contributors.
