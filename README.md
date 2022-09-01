# cargo-changelog

Changelog management tool for CLI.

`cargo-changelog` is a merge-friendly changelog management utility.

## Usage

Changelogs are (by default) created with `cargo-changelog new` in in
`/.changelogs/unreleased`, using a timestamp for their names.

Once you are done with one release, `cargo-changelog generate <version>` (for
example `cargo changelog generate minor` for the next minor version) will take
all unreleased changelogs and move them to `/.changelogs/0.1.0` (if "0.1.0" is
your next minor version - you can of course also specify an explicit version
with the `generate` subcommand).
After that you can create your final `CHANGELOG.md` file using
`cargo-changelog release`.

You can configure `cargo-changelog` in `/.changelog.toml` and add mandatory or
optional metadata fields to your changelog entries. You can also specify a
template file that gets used when rendering your changelogs to your final
`CHANGELOG.md` file.

## Current state

This project is in pre-alpha.

Please feel free to play with it, but do not consider anything stable yet!

## License

(c) 2022 Matthias Beyer
License: GPL-2.0-only
