<div align="center">
  <img src="docs/assets/streamfy-logo.png" alt="Streamfy" width="400" />
  <h1>Streamfy</h1>
  <p><em>A fork of <a href="https://github.com/fluvio-community/fluvio">Fluvio</a> by InfinyOn</em></p>
  <!-- <br> -->
  <!-- <br> -->

<!-- [![CI Status](https://github.com/streamfy-io/streamfy/actions/workflows/hourly.yml/badge.svg)](https://github.com/streamfy-community/streamfy/actions/workflows/hourly.yml) -->
<!--   [![CD Status](https://github.com/streamfy-io/streamfy/workflows/CD_Dev/badge.svg)](https://github.com/streamfy-community/streamfy/actions/workflows/cd_dev.yaml) -->
<!--   [![streamfy Crates.io version](https://img.shields.io/crates/v/streamfy?style=flat)](https://crates.io/crates/streamfy) -->
<!--   [![Streamfy Rust documentation](https://docs.rs/streamfy/badge.svg)](https://docs.rs/streamfy) -->
<!--   [![Streamfy dependency status](https://deps.rs/repo/github/streamfy-io/streamfy/status.svg)](https://deps.rs/repo/github/streamfy-community/streamfy) -->
<!--   [![Streamfy Discord](https://img.shields.io/discord/695712741381636168.svg?logo=discord&style=flat)](https://discordapp.com/invite/bBG2dTz) -->

<!-- <br> -->

  <!-- <br> -->
</div>

## What is Streamfy?

**Streamfy** is a distributed streaming engine plataform written in Rust. 

## Quick Start - Get started with Streamfy in 2 minutes or less!

### Step 1. Download Streamfy Version Manager:

Streamfy is installed via the **Streamfy Version Manager**, shortened to `svm`.

To install `svm`, run the following command:

```bash
curl -fsS https://raw.githubusercontent.com/streamfy-io/streamfy/main/install.sh | bash
```

As part of the initial setup, `svm` also installs the Streamfy CLI. When only the rolling development release is available, that is the **`latest`** channel (GitHub `dev` tag, version `VERSION+commit`). Once stable releases are published, the installer prefers **`stable`**.

Streamfy is stored in `$HOME/.streamfy`, with the executable binaries stored in `$HOME/.streamfy/bin`.

After install, ensure `~/.svm/bin` and `~/.streamfy/bin` are on your `PATH` (the installer prints shell-specific hints).

### Step 2. Start a cluster:

Start cluster on you local machine with the following command:

```bash
streamfy cluster start
```

### Step 3. Create Topic:

The following command will create a topic called hello-streamfy:

```bash
streamfy topic create hello-streamfy
```

### Step 4. Produce to Topic, Consume From Topic:

Produce data to your topic. Run the command first and then type some messages:

```bash
streamfy produce hello-streamfy
> hello streamfy
Ok!
> test message
Ok!
```

Consume data from the topic, Run the following command in a different terminal:

```bash
streamfy consume hello-streamfy -B -d
```

Just like that! You have a local cluster running.

<!-- ## Contributing -->
<!---->
<!-- If you'd like to contribute to the project, please read our -->
<!-- [Contributing guide](CONTRIBUTING.md). -->
<!---->
<!-- ### Contributors are awesome -->
<!-- <a href="https://github.com/streamfy-io/streamfy/graphs/contributors"> -->
<!--   <img src="https://contrib.rocks/image?repo=streamfy-io/streamfy" /> -->
<!-- </a> -->
<!---->
<!-- Made with [contrib.rocks](https://contrib.rocks). -->

## License

This project is licensed under the [Apache license](LICENSE).
