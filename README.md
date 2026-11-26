# Amanita

Open-source gaming mouse management software for Linux.

> [!IMPORTANT]
> **Early Development**
>
> Amanita is still under development and is not yet a functional application.

## Development

The initial supported build platform is Linux x86-64 (`x86_64-linux`).

Nix supplies the development tools and dependencies.
[Install Nix](https://nixos.org/download/) before you start working on the project.
Then enable `nix-command` and `flakes` in your Nix configuration:

```text
experimental-features = nix-command flakes
```

Enter the development environment:

```sh
nix develop
```

The project uses a Makefile as a simple wrapper around Cargo commands and the
tools provided by Nix. The same Make targets are used in CI and can also be used
during development.

Available `make` commands inside `nix develop`:

| Command | Description |
| --- | --- |
| `make` / `make build-debug` | Build the debug executable |
| `make build-release` | Build the release executable |
| `make check` | Check formatting, linting and CI workflows |
| `make test` | Run Cargo unit tests |
| `make format` | Apply Rust and Nix formatting |
| `make package` | Create an archive of the release executable |
| `make clean` | Remove the entire local `output/` directory |

The Makefile uses `--locked` for builds, tests and Clippy, so these commands fail
if `Cargo.lock` needs to change. After changing dependencies in `Cargo.toml`, run
`cargo check` without `--locked` to update the lockfile. Commit the updated
`Cargo.lock` with the dependency changes.

`make package` archives Cargo's release executable and the license text.
After changing target or linker settings, run `make clean` before
rebuilding to avoid reusing old build artifacts.

Generated files are kept in the `output/` directory. Cargo uses
`output/target/` and package archives use `output/dist/`.

## CI

CI uses one job with sequential stages, sharing the runner and Nix store:

| Stage | Purpose |
| --- | --- |
| `prepare` | Install Nix and load the locked development environment |
| `check` | Check formatting, linting and CI workflows |
| `unit test` | Cargo unit tests |
| `build` | Build the release executable |
| `smoke test` | Run the executable without Nix or system libraries |
| `package` | Archive the release executable |
| `deploy` | Upload the package artifact |

The smoke test runs the release executable with `chroot` in an empty root
filesystem to check for missing runtime dependencies at startup. A nonzero exit
code or timeout fails CI.

## License

Amanita is free software licensed under the GNU General Public License,
version 3 or any later version (`GPL-3.0-or-later`). See [LICENSE](LICENSE)
for the full license text.
