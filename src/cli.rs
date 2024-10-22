use clap::{ArgAction, Parser};

#[derive(clap::ValueEnum, Clone, Copy, Default, Debug)]
pub(crate) enum TimeStyle {
    #[default]
    Default,
    Iso,
    Relative,
}

#[derive(Debug, Default, Parser)]
#[clap(version, about = "A very basic ls clone")]
pub(crate) struct Args {
    #[clap(default_value = ".", help = "List of files/directories")]
    pub(crate) paths: Vec<String>,

    #[clap(
        short('a'),
        long("all"),
        action = ArgAction::Count,
        default_value_t = 0,
        help = "Show hidden and 'dot' files.\nUse this twice to also show '.' and '..' directories"
    )]
    pub(crate) all: u8,

    #[clap(
        short('l'),
        long("long"),
        default_value_t = false,
        help = "Display extended file metadata as a table"
    )]
    pub(crate) long: bool,

    #[clap(
        short('B'),
        long("bytes"),
        default_value_t = false,
        help = "List file sizes in bytes, without any prefixes"
    )]
    pub(crate) bytes: bool,

    #[clap(
        short('D'),
        long("only-dirs"),
        default_value_t = false,
        help = "List only directories"
    )]
    pub(crate) only_dirs: bool,

    #[clap(
        short('f'),
        long("only-files"),
        default_value_t = false,
        help = "List only files"
    )]
    pub(crate) only_files: bool,

    #[clap(
        short('g'),
        long("group"),
        default_value_t = false,
        help = "List each file's group"
    )]
    pub(crate) group: bool,

    #[clap(
        short('i'),
        long("inode"),
        default_value_t = false,
        help = "List each file's inode number"
    )]
    pub(crate) inode: bool,

    #[clap(
        short('H'),
        long("links"),
        default_value_t = false,
        help = "List each file's number of hard links"
    )]
    pub(crate) links: bool,

    #[clap(
        short('1'),
        long("oneline"),
        default_value_t = false,
        help = "Display one entry per line"
    )]
    pub(crate) oneline: bool,

    #[clap(
        long("no-permissions"),
        default_value_t = false,
        help = "Suppress the permissions field"
    )]
    pub(crate) no_permissions: bool,

    #[clap(
        long("time-style"),
        default_value = "default",
        ignore_case = true,
        help = "Time format"
    )]
    pub(crate) time_style: TimeStyle,

    #[clap(
        short('@'),
        long("extended"),
        default_value_t = false,
        help = "list each file's extended attributes"
    )]
    pub(crate) extended: bool,
}
