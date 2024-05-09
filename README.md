A toy project to mimic `ls` command in Rust. Currently, only `-a` and `-l` options are available.

```text
$ lsr -h
A very basic ls clone

Usage: lsr [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...  [default: .]

Options:
  -l, --long      Long listing
  -a, --show-all  Show all files
  -h, --help      Print help
  -V, --version   Print version
```