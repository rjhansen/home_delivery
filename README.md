# Description

`home_delivery` was written to practice my Rust skills in a socially
useful way, by creating a stable binary to replace a PowerShell
script that was getting a little creaky what with the changes to
PowerShell over the years.

The task is simple and well-defined. At the command line the user
specifies a source and destination directory. `home_delivery` will then
loop, beginning at startup and at the top of each minute thereafter,
looking in `srcdir` for any file that looks like, e.g., `3d0900.txt`.
In this case, that filename means “deliver this file to `dstdir` on the
third day at 0900.” Once that time marker is hit, the file is copied
over and the original deleted.

# Additional features

`home_delivery` uses `log4rs` to do its logging, and this must be
configured via a YAML file. A useful sample one is included in this
repository as `config.yaml`.

# License

Apache 2.0. Share and enjoy.
