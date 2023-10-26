<div align="center">

# 📊 `ds` - Real-time Stats with Terminal Charts

*Visualize container stats with beautiful, real-time charts directly in your terminal.*

  <img src="./assets/demo.png" alt="demo" />
</div>

> [!IMPORTANT]
> This is a WIP, `main` should be stable, but keep in mind this is changing constantly. Thanks! :)

## Why `ds`?

- **Missing Charts**: While experimenting with [OrbStack](https://orbstack.dev/), a lightweight Docker container management tool, I found that it lacks the visual charts that Docker Desktop provides. This project aims to bridge that gap.
- **Visual Analysis**: Visualizing system stats in real-time can help spot issues that might go unnoticed in text-based outputs. `ds` brings colors and moving charts to your system analysis.
- **Rust-Powered 😶‍🌫️**: This project is written in Rust, leveraging its performance and reliability.

## Installation

> [!NOTE]
> I plan to publish `ds` on [crates.io](https://crates.io/) once it's stable enough.

### Source

Ensure you have [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed. Then, clone this repo:

```bash
git clone https://github.com/rafaelrcamargo/ds
```

#### Development

Then just `cd` into the directory and run:

```bash
cargo run -- <ARGS> # For development
```

#### Release

In the project directory, run:

```bash
cargo run --release -- <ARGS> # For production
```

Then add this command to your `$PATH`, you can run:

```bash
# May require sudo
mv target/release/ds /usr/local/bin
```

## Usage

To use it with the default settings, just run:

```bash
ds
```

> This is akin to docker stats.

For more options, you can run:

```bash
ds -h
```

### Examples & Use Cases

Some examples of how you can use this tool.

#### Compact view for all containers

For a basic overview of all containers in a space-saving format:

```bash
ds -c
```

#### Full view for some containers

To see detailed stats for a specific container, including NET and IO charts:

```bash
ds -f 5f03524a8fbe api-1
```

## Roadmap

- [x] https://github.com/rafaelrcamargo/ds/issues/2
- [x] https://github.com/rafaelrcamargo/ds/issues/5
- [ ] https://github.com/rafaelrcamargo/ds/issues/8

## Pain Points

Some things that are bad, but expected.

<details open>
<summary>

### Painfully slow to start

</summary>

I know... It is slow to start, but that's it. This is the time `docker stats` takes to run, it has to hook up to the container and get the realtime stats. As for today I have tested it with OrbStack and Docker Desktop, the delay seems to be the same, but I'll keep looking into it.

</details>

<details>
<summary>

### `network_mode: host`

</summary>

From the GIF you can also note that the `NET` chart is not moving, but this is expected there. This containers are running in `network_mode: host` and the `NET` chart will only be populated if you're using the `bridge` network.

> Ps: If you use Mac and think I'm completely out of my mind for the `network_mode: host` above, I know. It's a running topic on the **Docker Desktop for Mac** and yet not supported. You can follow the discussion [here](https://github.com/docker/roadmap/issues/238). And this was the main reason I started looking into OrbStack.

</details>

## License

This project is licensed under the [Apache License 2.0](LICENSE).
