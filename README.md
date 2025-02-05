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

# Usage
```bash
$ home_delivery --source SRCDIR --destination DESTDIR --config CONFIG_FILE
```

Windows users will want to add a `.exe` to the end of the application
name. The application also supports `--help` and `--version` flags.

## The config file
`home_delivery` supports logging via log4rs. Logging is user-configurable
via a YAML-formatted configuration file. One is provided in the code
repository as `config.yaml`, but if for some reason you can’t access that,
it’s recreated here:

```yaml
appenders:
  screen:
    kind: console
    encoder:
      pattern: "{h({d(%Y-%m-%dT%H:%M:%SZ)(utc)} - {l}: {m}{n})}"
  file:
    kind: rolling_file
    path: "log/home_delivery.txt"
    encoder:
      pattern: "{d(%Y-%m-%dT%H:%M:%SZ)(utc)} - {h({l})}: {m}{n}"
    policy:
      trigger:
        kind: size
        limit: 50kb
      roller:
        kind: delete
root:
  level: info
  appenders:
    - screen
    - file
```

# Time format
All days are zero-indexed. Today is day zero of the exercise:
tomorrow is day one. A filename may omit the `Xd` prefix, in which
case it's assumed X is zero.

All times are in UTC. A file named `0800.txt` will be delivered on
day zero at 0800 UTC. One named `3d1500.pdf` will be delivered on day
three at 1500 UTC.

# Bugs
Bugs should go to the 
[bug tracker](https://github.com/rjhansen/home_delivery/issues) but,
if you can’t access GitHub where you are, may be
[emailed to me](mailto:rob@hansen.engineering?subject=home_delivery%20bug)
directly.

# License
[Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0.html). Share and enjoy.
