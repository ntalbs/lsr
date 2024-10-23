A toy project to mimic `ls` command in Rust. Currently, only small subsets of the options are available.

```text
$ lsr --help
A very basic ls clone

Usage: lsr [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...  List of files/directories [default: .]

Options:
  -a, --all...                   Show hidden and 'dot' files.
                                 Use this twice to also show '.' and '..' directories
  -x, --across                   Sort the grid across, rather than downwards
  -l, --long                     Display extended file metadata as a table
  -B, --bytes                    List file sizes in bytes, without any prefixes
  -D, --only-dirs                List only directories
  -f, --only-files               List only files
  -g, --group                    List each file's group
  -i, --inode                    List each file's inode number
  -H, --links                    List each file's number of hard links
  -1, --oneline                  Display one entry per line
      --no-permissions           Suppress the permissions field
      --time-style <TIME_STYLE>  Time format [default: default] [possible values: default, iso, relative]
  -@, --extended                 list each file's extended attributes
  -h, --help                     Print help
  -V, --version                  Print version
```
