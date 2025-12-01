# Sundial

Display astronomical events using a raspberry pi pico and an e-paper display.

## Running the project

The project is structured as a cargo workspace with two binary packages,
`sundial` (which runs on the raspberry pi), and `simulator` (which will
eventually run similar code on a developer machine). There's probably a good
way to have each of those packages use different default targets -- but I don't
know what that way is, so for now there's a Makefile you can use to run either
on the raspberry pi:

```Makefile
make run-sundial
```

or locally:

```Makefile
make run-simulator
```

