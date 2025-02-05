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
third day at 0900 UTC.” Once that time marker is hit, the file is 
copied over and the original deleted.

# Time format
All days are zero-indexed. Today is day zero of the exercise:
tomorrow is day one. A filename may omit the `Xd` prefix, in which
case it's assumed X is zero.

All times are in UTC. A file named `0800.txt` will be delivered on
day zero at 0800 UTC. One named `3d1500.pdf` will be delivered on day
three at 1500 UTC.

# Additional features
`home_delivery` uses `log4rs` to do its logging, and this must be
configured via a YAML file. A useful sample one is included in this
repository as `config.yaml`.

# License
Apache 2.0. Share and enjoy.
