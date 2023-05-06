# fimfic-tracker

An unnecessary CLI application for tracking [Fimfiction][fimfiction] stories.

## Why?

This could be useful to you if:

* You don't have and/or don't want to create a Fimfiction account to track stories.
* You want to have an up-to-date local copy, be it for archiving purposes or for offline reading.
* You find yourself using programs like [FanFicFare] or [fimfic2epub] as an alternative for
  reading stories you follow (see [Alternative download method](#alternative-download-method)).
* You just want an excuse to not leave the terminal as often.

So, if you have a Fimfiction account and are content with reading online this might not be for you.

## Usage

### Tracking stories

First thing to do is add stories to the tracking list, since at the start this will be empty,
and for that you use the `track` command like so:

<!-- CHECK: Tracking with an URL -->
```sh
  fimfic-tracker track "https://www.fimfiction.net/story/000000/a-story-title"
```

You can also add multiple stories to the tracking list at the same time

<!-- CHECK: Tracking with two URLs -->
```sh
  fimfic-tracker track "https://www.fimfiction.net/story/000001/another-story-with-ponies" \
    "https://www.fimfiction.net/story/000002/story-with-bug-ponies"
```

The default behavior is to download a story after adding them to the tracking list, if you prefer
to skip that you can use the `--skip-download` flag.

<!-- CHECK: Skip download while tracking -->
```sh
  fimfic-tracker track --skip-download \
    "https://www.fimfiction.net/story/000002/story-with-bug-ponies"
```

The argument can be the URL to the main story page, the URL to a chapter and even the story ID if
you find yourself unable to copy it. For instance, all of the following arguments are valid.

<!-- CHECK: Valid arguments for stories -->
```sh
  fimfic-tracker track \
    "https://www.fimfiction.net/story/000003/2/a-story-with-bird-ponies/chapter-2" \
    "https://www.fimfiction.net/story/000004/a-pie-story" \
    "https://www.fimfiction.net/story/000005" \
    000006
```

### Looking at the list

You can see your current list at any time via the `list` command, giving you the story IDs and
most of the information stored in the actual tracker file.

<!-- CHECK: List tracked stories -->
```sh
  fimfic-tracker list
```

With the `--short` flag the output is reduced to one line per story, ID and title.

<!-- CHECK: List only IDs and titles of tracked stories -->
```sh
  fimfic-tracker list --short
```

This is useful to get the ID related to a specific story for using while
[checking for updates](#checking-for-updates) and [untracking stories](#untracking-stories).

### Checking for updates

Now with some stories in the list, sometime later you would come back and use the `download` command.

<!-- CHECK: Check for updates -->
```sh
  fimfic-tracker download
```

This will go through the tracking list in order, checking for updates and then download the ones that
did update while keeping the list up-to-date. By default, the application will download stories whose
amount of chapters has changed in the HTML format provided by Fimfiction into your downloads directory
(see [Configuration](#configuration)).

If you feel the need to do this in only a few of them, you can also supply a story URL or ID.

<!-- CHECK: Check for updates on specific stories -->
```sh
  fimfic-tracker download 000000 000002
```

Or if you want to force a download even if there are no updates you would use the `--force` flag.

<!-- CHECK: Force download a story -->
```sh
  fimfic-tracker download --force 000004
```

### Untracking stories

Once you want to stop tracking a story, be it by reaching completion or just losing interest, you use
the aptly named `untrack` command

<!-- CHECK: Untrack stories -->
```sh
  fimfic-tracker untrack 000000 000001
```

## Configuration

The application loads the configuration from different sources in a specific order with a sensible
option overriding behavior that you would expect from this system.

1. Default configuration.

2. User configuration file, if it exists.
    * On Windows, `C:\Users\USER\AppData\Local\Fimfiction Tracker\config\config.toml`.
    * On Mac, `/Users/USER/Library/Application Support/Fimfiction-Tracker/config.toml`.
    * On Linux, `/home/USER/.config/fimfic-tracker/config.toml`.

3. Environment variables.

4. Configuration file provided by `--config`, if used.

For documentation on the different options and their respective environment variables refer to the
[example](/core/config/default.toml) that you can take as a starting point.

### The `exec` option

This is the more "hackable" configuration option, it allows you to override the default download
with the running of any executable that you have in your system. Allowing you to effectively
hook up any process started by the update of a story, the only limit of what can be done is your creativity.
Here are some examples of what can be done with it.

#### Alternative download method

If the default download formats are not to your liking, you can make another executable do the
actual download.

* [`fimfic2epub`][fimfic2epub]
```toml
exec = "fimfic2epub --dir $DOWNLOAD_DIR $ID"
```

* [`FanFicFare`][FanFicFare]
```toml
exec = 'fanficfare --format=$FORMAT --non-interactive --option output_filename="$DOWNLOAD_DIR/$${title}-$${siteabbrev}_$${storyId}$${formatext}" $URL'
```

#### Simple archive.

A simple archive with the content as HTML and the JSON API response of a story can be achieved by using
a custom script.

```bash
#!/bin/bash

set -e

ID=$1
FORMAT=$2
DOWNLOAD_DIR=$3

([ -n "$ID" ] && [ -n "$FORMAT" ] && [ -n "$DOWNLOAD_DIR" ]) || exit 1

wget -q -O "$DOWNLOAD_DIR/$ID.json" "https://www.fimfiction.net/api/story.php?story=$ID"
wget -q -O "$DOWNLOAD_DIR/$ID.$FORMAT" "https://www.fimfiction.net/story/download/$ID/$FORMAT"
```

If the path to this script is `/path/to/fft-download-script`, then `exec` would end up as.

```toml
exec = "/path/to/fft-download-script $ID $FORMAT $DOWNLOAD_DIR"
```

Don't forget to make it executable.

```sh
  chmod +x /path/to/fft-download-script
```

#### Notifications

You can discard making a download entirely and do something related to having an update, like having
system notifications via `notify-send`.

```toml
exec = "notify-send -u normal 'A story has updated' '<span weight=\"bold\">$TITLE</span> got an update'"
```

## Building

Firstly, Rust should be installed in your system. Instructions on how to do so can be found [on its website](https://www.rust-lang.org/tools/install).

Then you download this repository or clone it using `git`.

```sh
  git clone https://github.com/ZodiacalComet/fimfic-tracker.git
  cd fimfic-tracker
```

And then you build the executable.

```sh
  cargo build --release
```

The resulting binary will be located in `target/releases`.

## Acknowledgments

To these projects that made this one easier to get done!

* [chrono], a date and time library for Rust.
* [chrono-humanize], human-friendly time expressions - similar to Python arrow.humanize.
* [clap], a simple to use, efficient, and full-featured Command Line Argument Parser.
* [clap_complete], shell completion generation for clap.
* [console], a terminal and console abstraction for Rust.
* [dialoguer], a command line prompting library.
* [directories], a mid-level library that provides config/cache/data paths, following the respective conventions on Linux, macOS and Windows.
* [env_logger], a logging implementation for `log` which is configured via an environment variable.
* [envy], deserialize env vars into typesafe structs,
* [eyre], a trait object based error handling type for easy idiomatic error handling and reporting in Rust applications.
* [futures-util], common utilities and extension traits for the futures-rs library.
* [indexmap], a hash table with consistent order and fast iteration.
* [lazy_static], a small macro for defining lazy evaluated static variables in Rust.
* [log], a Rust library providing a lightweight logging facade.
* [number_prefix], a library for numeric prefixes, such as "Kilo" or "Giga" or "Kibi".
* [reqwest], an easy and powerful Rust HTTP Client.
* [serde], a framework for serializing and deserializing Rust data structures efficiently and generically.
* [serde_json], a strongly typed JSON library for Rust.
* [shellexpand], a library for shell-like expansion in strings.
* [shlex], split a string into shell words, like Python's shlex.
* [tokio], an event-driven, non-blocking I/O platform for writing asynchronous I/O backed applications.
* [toml], a serde-compatible TOML decoder and encoder for Rust.
* [url], an URL library for Rust, based on the WHATWG URL Standard.

## Contribute

You can easily contribute by reporting unexpected errors to proposing new features that you feel would be useful to
have embedded into the program itself. Or get more involved by fixing typos to improving the `core` documentation,
program messages and/or this README.

You can also contribute code if you want to, though if it is for implementing a new feature I would recommended you to
[open an issue][issues] first before trying to work on it, I would hate for you to waste your time if it doesn't end up
merged for any reason.

## License

Distributed under the [Unlicense License](LICENSE).

[fimfiction]: https://www.fimfiction.net/
[FanFicFare]: https://github.com/JimmXinu/FanFicFare
[fimfic2epub]: https://github.com/daniel-j/fimfic2epub

[issues]: https://github.com/ZodiacalComet/fimfic-tracker/issues

[chrono]: https://github.com/chronotope/chrono
[chrono-humanize]: https://gitlab.com/imp/chrono-humanize-rs
[clap]: https://github.com/clap-rs/clap
[clap_complete]: https://github.com/clap-rs/clap/tree/master/clap_complete
[console]: https://github.com/console-rs/console
[dialoguer]: https://github.com/console-rs/dialoguer
[directories]: https://github.com/dirs-dev/directories-rs
[env_logger]: https://github.com/rust-cli/env_logger
[envy]: https://github.com/softprops/envy
[eyre]: https://github.com/yaahc/eyre
[futures-util]: https://github.com/rust-lang/futures-rs
[indexmap]: https://github.com/bluss/indexmap
[lazy_static]: https://github.com/rust-lang-nursery/lazy-static.rs
[log]: https://github.com/rust-lang/log
[number_prefix]: https://github.com/ogham/rust-number-prefix
[reqwest]: https://github.com/seanmonstar/reqwest
[serde]: https://github.com/serde-rs/serde
[serde_json]: https://github.com/serde-rs/json
[shellexpand]: https://gitlab.com/ijackson/rust-shellexpand
[shlex]: https://github.com/comex/rust-shlex
[tokio]: https://github.com/tokio-rs/tokio
[toml]: https://github.com/toml-rs/toml
[url]: https://github.com/servo/rust-url
