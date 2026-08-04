# The Observable Café

A little playground for explaining metric types: buying a coffee increments a
counter, while two temperature gauges drift up and down on their own.

The café lives on the server. The browser only draws what it is told, so every
visitor sees the same numbers — and so does anything scraping `/metrics`.

Built with [Dioxus](https://dioxuslabs.com).

## Development

```shell
dx serve
```

Then open <http://localhost:8080>, and <http://localhost:8080/metrics> for the
scrape endpoint.

```shell
dx build --release   # bundles into target/dx/observable-cafe/release/web/public
cargo clippy --no-default-features --features server
cargo clippy --target wasm32-unknown-unknown --no-default-features --features web
cargo fmt
```

The `web` and `server` features pick which half of the app is being built; `dx`
sets them for you.

## Layout

| Path                       | Contents                                            |
| -------------------------- | --------------------------------------------------- |
| `src/app.rs`               | Root component: polling, page layout                |
| `src/components/`          | Menu, thermometers, notebook, sparkline, toast      |
| `src/state.rs`             | What the server observes and hands to the browser   |
| `src/season.rs`            | What the calendar says a thermometer should read    |
| `src/api.rs`               | Server functions the browser calls                  |
| `src/server.rs`            | The café: in-memory state and the axum router       |
| `src/server/simulation.rs` | Readings that move on their own                     |
| `src/server/metrics.rs`    | The `/metrics` scrape endpoint                      |
| `src/server/rng.rs`        | Seeded xorshift used to simulate readings           |
| `assets/main.css`          | Styles                                              |

## Metrics

```
cafe_coffees_sold_total            counter, reset by the reset button
cafe_inside_temperature_celsius    gauge
cafe_outside_temperature_celsius   gauge
```
