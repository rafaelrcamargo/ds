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

### Usage

Well, there's not much to it. Just run `ds` and you should see *colored moving things* in your terminal.

```bash
ds
```

### License

This project is licensed under the [Apache License 2.0](LICENSE).
