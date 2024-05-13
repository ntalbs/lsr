A toy project to mimic `ls` command in Rust. Currently, only `-a` and `-l` options are available.

```text
$ lsr -h
A very basic ls clone

Usage: lsr [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...  List of files/directories [default: .]

Options:
  -l, --long     Show hidden and 'dot' files including '.' and '..' directories
  -a, --all      Display extended file metadata as a table
  -B, --bytes    List file sizes in bytes, without any prefixes
  -g, --group    List each file's group
  -h, --help     Print help
  -V, --version  Print version
```
