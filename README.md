# The Observable Café

A little playground for explaining metric types: buying a coffee increments a
counter, while two temperature gauges drift up and down on their own.

Built with [Dioxus](https://dioxuslabs.com). Currently a web-only client app;
server functions will come later.

## Development

```shell
dx serve
```

Then open <http://localhost:8080>.

```shell
dx build --release   # bundles into target/dx/observable-cafe/release/web/public
cargo clippy --target wasm32-unknown-unknown
cargo fmt
```

## Layout

| Path              | Contents                                             |
| ----------------- | ---------------------------------------------------- |
| `src/app.rs`      | Root component: state, the reading loop, page layout |
| `src/components/` | Menu, thermometers, notebook, sparkline, toast       |
| `src/state.rs`    | `Gauge`: a reading, its history and its scale        |
| `src/rng.rs`      | Seeded xorshift used to simulate readings            |
| `assets/main.css` | Styles                                               |
