A toy project to mimic `ls` command in Rust. Currently, only small subsets of the options are available.

```text
$ lsr -h
A very basic ls clone

Usage: lsr [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...  List of files/directories [default: .]

Options:
  -l, --long            Show hidden and 'dot' files including '.' and '..' directories
  -a, --all             Display extended file metadata as a table
  -B, --bytes           List file sizes in bytes, without any prefixes
  -D, --only-dirs       List only directories
  -f, --only-files      List only files
  -g, --group           List each file's group
  -1, --oneline         Display one entry per line
      --no-permissions  Suppress the permissions field
  -h, --help            Print help
  -V, --version         Print version
```
