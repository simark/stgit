# Stacked Git

Stacked Git, **StGit** for short, is an application for managing Git
commits as a stack of patches.

With a *patch stack* workflow, multiple patches can be developed
concurrently and efficiently, with each patch focused on a single
concern, resulting in both a clean Git commit history and improved
productivity.

For a complete introduction to StGit, see the [Stacked Git
homepage](https://stacked-git.github.io).

## Local modifications

This branch carries a few patches on top of upstream StGit, all of them
adding configuration options for how the patch series is displayed. See
[`Documentation/stg.txt`](Documentation/stg.txt) for the reference
documentation of each option.

- `stgit.series.top-first`: a boolean that makes commands which list the
  patch series, namely `stg series` and `stg patches`, display the
  topmost patch first instead of the bottommost patch first. Defaults to
  `false`. With it enabled, `stg series --reverse` restores the
  bottom-first order. Commands whose ordering is functional rather than a
  listing, such as `stg export`, `stg email` and `stg push`, are left
  alone.

- `stgit.series.show-after-change`: a boolean that makes commands which
  change which patches are applied, such as `stg push`, `stg pop` and
  `stg goto`, display the resulting series instead of just the patches
  they pushed or popped. Defaults to `false`. The series is rendered like
  `stg series`, and patches pushed by the command keep their status
  annotation, so a halted push still shows which patch conflicted, in the
  context of the whole series. Commands that do not change which patches
  are applied, such as `stg refresh` and `stg rename`, are unaffected.

- `stgit.series.description`: a boolean that makes `stg series` annotate
  each patch with its short description, as if `--description` had been
  given. Defaults to `false`, and `--description`/`--no-description` still
  take precedence over it. It also applies to the series printed by
  `stgit.series.show-after-change`. This restores an option of the same
  name that existed in the Python implementation of StGit.

## Getting started

To get a feel for how StGit works, see this brief [example of StGit in
action][example]. Or check out the [in-depth tutorial][tutorial].

[example]: https://stacked-git.github.io/guides/usage-example
[tutorial]: https://stacked-git.github.io/guides/tutorial

StGit also has a complete set of [man pages][man] describing the
[`stg`][stg] command line tool and each of its subcommands.

[man]: https://stacked-git.github.io/man
[stg]: https://stacked-git.github.io/man/stg

## Installation

See [`CHANGELOG.md`](CHANGELOG.md) to see what has changed in the latest
StGit release.

### Dependencies

StGit is implemented in Rust using a number of third-party, open source
crates. StGit statically links with its pure-Rust dependencies, but
dynamically links to libc and other non-Rust libraries when they are
available at build-time. Dynamic link dependencies include these
libraries along with their transient link dependencies:

- libcurl (optional)

StGit works within the context of a Git repository and performs many
operations by running subordinate `git` commands.
[Git](https://git-scm.com) 2.2.0 or newer is required.

### Package Repositories

Recent versions of StGit are available in several package repositories
such as [HomeBrew][pkg-homebrew] and [MacPorts][pkg-macports] for MacOS
and for the [Arch][pkg-arch] and [Gentoo][pkg-gentoo] Linux
distributions. StGit is also available via [crates.io][pkg-crate],
[guix][pkg-guix], and [nix][pkg-nix].

More details about StGit packages availability for various operating
systems can be [found on repology][repology].

[pkg-homebrew]: https://formulae.brew.sh/formula/stgit
[pkg-macports]: https://ports.macports.org/port/stgit/
[pkg-arch]: https://aur.archlinux.org/packages/stgit
[pkg-gentoo]: https://packages.gentoo.org/packages/dev-vcs/stgit
[pkg-crate]: https://crates.io/crates/stgit
[pkg-guix]: https://packages.guix.gnu.org/packages/stgit/
[pkg-nix]: https://search.nixos.org/packages?type=packages&query=stgit
[repology]: https://repology.org/project/stgit/versions

### Prebuilt Packages

Prebuilt deb, rpm, and msi packages are provided by the StGit project.
Packages for the latest release may be found [here][latest].

Note that the Linux deb and rpm packages are unofficial. The upstream
Debian and RedHat/Fedora projects currently only publish outdated
versions of StGit (see [repology][repology]). These unofficial packages
are meant to be a stop-gap until official StGit packages are provided by
downstream distributions.

The Linux deb and rpm packages are statically linked use [`musl`][musl]
libc to maximize compatibility. They should hopefully work on a wide
range of deb and rpm based distributions.

[musl]: https://musl.libc.org/

### Source Installation

StGit may also be installed from source. Download the [latest
release][latest] or clone from the [StGit repository on GitHub][repo].

[latest]: https://github.com/stacked-git/stgit/releases/latest
[repo]: https://github.com/stacked-git/stgit

To install the `stg` executable from source, choose a `prefix` and run:

```shellsession
$ make prefix=$HOME/.local install
```

For more information about installation, see [`INSTALL.md`](INSTALL.md).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for a full guide to contributing
to StGit.

## Maintainers

StGit is maintained by Catalin Marinas and Peter Grayson.

For a complete list of StGit's authors, see [`AUTHORS.md`](AUTHORS.md).
