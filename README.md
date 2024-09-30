A toy project to mimic `ls` command in Rust. Currently, only small subsets of the options are available.

```text
$ lsr --help
A very basic ls clone

Usage: lsr [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...  List of files/directories [default: .]

Options:
  -l, --long                     Display extended file metadata as a table
  -a, --all                      Show hidden and 'dot' files including '.' and '..' directories
  -B, --bytes                    List file sizes in bytes, without any prefixes
  -D, --only-dirs                List only directories
  -f, --only-files               List only files
  -g, --group                    List each file's group
  -1, --oneline                  Display one entry per line
      --no-permissions           Suppress the permissions field
      --time-style <TIME_STYLE>  Time format [default: default] [possible values: default, iso, relative]
  -h, --help                     Print help
  -V, --version                  Print version
```
