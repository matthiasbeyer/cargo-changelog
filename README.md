# cargo-changelog

Changelog management tool for CLI.

`cargo-changelog` is a merge-friendly changelog management utility.

## Usage

`cargo-changelog` is a tool to generate and manage changelog entries.

It works in the following way:

- Everytime you add/change/fix something and want to document it, you create a
  new changelog entry with `cargo changelog new`
- Then, when a new version is released, you run `cargo changelog generate
  <bump>` to move all unreleased changes to either the next patch/minor/major
  version
- Finally, you re-generate the CHANGELOG.md file using `cargo changelog release`

Here's how they work individually:

### cargo changelog new

`cargo changelog new` generates a new changelog file in the unreleased
changelog directory. Per default that is `.changelogs/unreleased`.
If interactive mode is enabled, which it is per-default, then you will be
prompted to fill in the fields of the changelog as well as a larger free-form
entry where you can explain the motivation and consequences of the changes.

### cargo changelog generate <bump>

Once you are done with one release, `cargo-changelog generate <version>` (for
example `cargo changelog generate minor` for the next minor version) will take
all unreleased changelogs and move them to `/.changelogs/0.1.0` (if "0.1.0" is
your next minor version - you can of course also specify an explicit version
with the `generate` subcommand).

### cargo changelog release

After that you can create your final `CHANGELOG.md` file using
`cargo-changelog release`.

This will take all released changelog entries and generate a new file,
overwriting the old.

-------

## Configuration

You can configure `cargo-changelog` in `/changelog.toml` and add mandatory or
optional metadata fields to your changelog entries. You can also specify a
template file that gets used when rendering your changelogs to your final
`CHANGELOG.md` file.

## Current state

This project is in pre-alpha.

Please feel free to play with it, but do not consider anything stable yet!

## License

(c) 2022 Matthias Beyer
License: GPL-2.0-only
