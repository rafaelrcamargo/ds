# 📊 ds - Useful stats on your terminal

It's like "docker stats" but with beautiful, real-time charts directly in your terminal.

![ds](./assets/ds.gif)

## Why?

Lately I've been trying out [OrbStack](https://orbstack.dev/) - OrbStack is the fast, light, and easy way to run Docker containers and Linux. Develop at lightspeed with our Docker Desktop alternative.

And as much as I love the idea of OrbStack, I really miss the charts that **Docker Desktop** provides. This is a issue that's on the OrbStack team's radar, but I wanted to see if I could come up with a solution in the meantime.

I also think having a visual reference when analyzing your system can be very helpful. Colors and moving things can help you spot issues that you might not otherwise notice. And that's how this project was born.

## Installation

### Source

For this you'll need to have [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed.

```bash
git clone https://github.com/rafaelrcamargo/ds
```

`cd` into the directory and run:

```bash
cargo run # For development
cargo build --release # For production
```

To add this command to your path, you can run:

```bash
mv target/release/ds /usr/local/bin
```

> **Note**: Yes, I do plan on adding this to crates.io, but I want to make sure it's stable enough before I do that.

## Usage

Well, there's not much to it. Just run `ds` and you should see *colored moving things* in your terminal.

```bash
ds
```

## Pain Points

I know... It is slow to start, but that's it. This is the time `docker stats` takes to run, it has to hook up to the container and get the realtime stats. As for today I have tested it with OrbStack and Docker Desktop, the delay seems to be the same, but I'll keep looking into it.

From the GIF you can also note that the `NET` chart is not moving, but this is expected there. This containers are running in `network_mode: host` and the `NET` chart will only be populated if you're using the `bridge` network.

> Ps: If you use Mac and think I'm completely out of my mind for the `network_mode: host` above, I know. It's a running topic on the Desktop for Mac and yet not supported. You can follow the discussion [here](https://github.com/docker/roadmap/issues/238). And this was the main reason I started looking into OrbStack.

## License

This project is licensed under the [Apache License 2.0](LICENSE).
