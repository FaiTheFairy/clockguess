# To clone repo
```bash
git clone
cd clockguess
```

# To install
```bash
cargo install --path=.
clockguess --help
```
* Note: this requires `$HOME/.cargo/bin` to be in your PATH. Alternatively you could just run it (see below).

# To run without installing
```bash
cargo run -- --help
```
* Note: any application arguments are passed after the double dash after run. For example:
```bash
cargo run --release -- --theme unicode --mode rapid-fire
```
