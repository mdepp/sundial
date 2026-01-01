# Sundial

Display astronomical events using a raspberry pi pico and an e-paper display.

## Running the project

The project is structured as a cargo workspace with two binary packages,
`sundial` (which runs on the raspberry pi), and `simulator` (which runs similar
code on the local machine). There's probably a good way to have each of those
packages use different default targets -- but I don't know what that way is, so
for now there's a Makefile you can use to run either on the raspberry pi:

```Makefile
make run-sundial
```

or locally:

```Makefile
make run-simulator  # or simply "cargo run"
```

### Build environment, etc

The `rust-analyzer.cargo.target` key in `.vscode/settings.json` configures the
target that the vscode rust-analyzer extension uses to make its diagnostics.
By default this is set up for the simulator, which means it'll not necessarily
show errors correctly for the `sundial` package. So when you're editing the
`sundial` package, you probably want to temporarily uncomment that line:

```
  "rust-analyzer.cargo.target": "thumbv6m-none-eabi",
```
