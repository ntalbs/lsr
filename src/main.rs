use chrono::{DateTime, Local};
use clap::Parser;
use std::{
    fs::{self, Metadata},
    io::{self, Error},
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
    path::{Path, PathBuf},
};
use tabular::{Row, Table};
use term_grid::{Direction, Filling, Grid, GridOptions};
use terminal_size::{terminal_size, Width};
use users::{get_group_by_gid, get_user_by_uid};

#[derive(Debug, Default, Parser)]
#[clap(version, about = "A very basic ls clone")]
pub struct Args {
    #[clap(default_value = ".", help = "List of files/directories")]
    paths: Vec<String>,
    #[clap(
        short('l'),
        long("long"),
        default_value_t = false,
        help = "Show hidden and 'dot' files including '.' and '..' directories"
    )]
    long: bool,
    #[clap(
        short('a'),
        long("all"),
        default_value_t = false,
        help = "Display extended file metadata as a table"
    )]
    all: bool,
    #[clap(
        short('B'),
        long("bytes"),
        default_value_t = false,
        help = "List file sizes in bytes, without any prefixes"
    )]
    bytes: bool,
    #[clap(
        short('D'),
        long("only-dirs"),
        default_value_t = false,
        help = "List only directories"
    )]
    only_dirs: bool,
    #[clap(
        short('f'),
        long("only-files"),
        default_value_t = false,
        help = "List only files"
    )]
    only_files: bool,
    #[clap(
        short('g'),
        long("group"),
        default_value_t = false,
        help = "List each file's group"
    )]
    group: bool,
    #[clap(
        short('1'),
        long("oneline"),
        default_value_t = false,
        help = "Display one entry per line"
    )]
    oneline: bool,
    #[clap(
        long("no-permissions"),
        default_value_t = false,
        help = "Suppress the permissions field"
    )]
    no_permissions: bool,
}

fn file_type(path: &Path) -> String {
    if path.is_symlink() {
        "l".to_string()
    } else if path.is_dir() {
        "d".to_string()
    } else if path.is_file() {
        "-".to_string()
    } else {
        "?".to_string()
    }
}

fn format_mode(mode: u32) -> String {
    let mut perms = String::new();

    perms.push_str(if mode & 0b100000000 != 0 { "r" } else { "-" });
    perms.push_str(if mode & 0b010000000 != 0 { "w" } else { "-" });
    perms.push_str(if mode & 0b001000000 != 0 { "x" } else { "-" });
    perms.push_str(if mode & 0b000100000 != 0 { "r" } else { "-" });
    perms.push_str(if mode & 0b000010000 != 0 { "w" } else { "-" });
    perms.push_str(if mode & 0b000001000 != 0 { "x" } else { "-" });
    perms.push_str(if mode & 0b000000100 != 0 { "r" } else { "-" });
    perms.push_str(if mode & 0b000000010 != 0 { "w" } else { "-" });
    perms.push_str(if mode & 0b000000001 != 0 { "x" } else { "-" });

    perms
}

fn user_name(uid: u32) -> String {
    get_user_by_uid(uid)
        .map(|u| u.name().to_string_lossy().to_string())
        .unwrap_or_else(|| uid.to_string())
}

fn group_name(gid: u32) -> String {
    get_group_by_gid(gid)
        .map(|g| g.name().to_string_lossy().to_string())
        .unwrap_or_else(|| gid.to_string())
}

fn modified_date(md: &Metadata) -> String {
    let modified: DateTime<Local> = DateTime::from(md.modified().unwrap());
    modified.format("%Y-%m-%d %H:%M").to_string()
}

fn file_name(path: &Path, long: bool) -> String {
    if path == PathBuf::from(".") {
        return "./".into();
    } else if path == PathBuf::from("..") {
        return "../".into();
    }

    let mut name = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();
    if long && path.is_symlink() {
        if let Ok(target) = fs::read_link(path) {
            name.push_str(" -> ");
            name.push_str(&target.to_string_lossy());
        }
    } else if path.is_dir() {
        name.push('/');
    }
    name
}

fn file_size(md: &Metadata, bytes: bool) -> String {
    if !md.is_file() {
        return "-".into();
    }

    let len = md.len();
    if bytes {
        len.to_string()
    } else if len < 1024 {
        format!("{len}")
    } else if len < 1024 * 1024 {
        format!("{:.1}k", len as f64 / 1024.0)
    } else if len < 1024 * 1024 * 1024 {
        format!("{:.1}M", len as f64 / 1024.0 / 1024.0)
    } else {
        format!("{:.1}G", len as f64 / 1024.0 / 1024.0 / 1024.0)
    }
}

fn format_output_oneline(paths: &[PathBuf]) -> io::Result<String> {
    let mut output = String::new();
    for p in paths {
        output.push_str(&file_name(p, false));
        output.push('\n');
    }
    Ok(output)
}

fn format_output_short(paths: &[PathBuf]) -> io::Result<String> {
    let term_size = terminal_size();
    if let Some((Width(w), _)) = term_size {
        let cells = paths.iter().map(|p| file_name(p, false)).collect();
        let grid = Grid::new(
            cells,
            GridOptions {
                filling: Filling::Spaces(2),
                direction: Direction::TopToBottom,
                width: w as usize,
            },
        );
        Ok(format!("{grid}"))
    } else {
        Err(Error::new(
            io::ErrorKind::Other,
            "Failed to get terminal width.",
        ))
    }
}

fn format_output_long(paths: &[PathBuf], args: &Args) -> io::Result<String> {
    let fmt = "{:<}{:<}  {:>}  {:<}  {:<}  {:>}  {:<}  {:<}";
    let mut table = Table::new(fmt);

    for path in paths {
        let md = path.metadata()?;
        table.add_row(
            Row::new()
                .with_cell(file_type(path))
                .with_cell(if args.no_permissions { "".to_string() } else { format_mode(md.mode()) })
                .with_cell(md.nlink())
                .with_cell(user_name(md.uid()))
                .with_cell(if args.group { group_name(md.gid()) } else { "".to_string() })
                .with_cell(file_size(&md, args.bytes))
                .with_cell(modified_date(&md))
                .with_cell(file_name(path, true)),
        );
    }
    Ok(format!("{table}"))
}

fn files_in(path: &Path, args: &Args) -> io::Result<Vec<PathBuf>> {
    let mut results = vec![];
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let is_hidden = entry.file_name().as_os_str().as_bytes()[0] == b'.';

        if args.only_dirs ^ args.only_files {
            if args.only_dirs && path.is_file() {
                continue;
            }
            if args.only_files && path.is_dir() {
                continue;
            }
        }
        if args.all || !is_hidden {
            results.push(path);
        }
    }

    results.sort();

    if args.all {
        results.insert(0, PathBuf::from("."));
        results.insert(1, PathBuf::from(".."));
    }

    Ok(results)
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let mut paths = args
        .paths
        .iter()
        .map(PathBuf::from)
        .filter(|p| {
            if p.exists() {
                true
            } else {
                eprintln!("{}: No such file or directory.", file_name(p, false));
                false
            }
        })
        .collect::<Vec<PathBuf>>();

    paths.sort_by(|a, b| {
        if a.is_dir() && !b.is_dir() {
            std::cmp::Ordering::Greater
        } else if !a.is_dir() && b.is_dir() {
            std::cmp::Ordering::Less
        } else {
            a.cmp(b)
        }
    });

    let (files, directories): (Vec<_>, Vec<_>) = paths.iter().cloned().partition(|f| !f.is_dir());

    // print files first
    if args.oneline {
        print!("{}", format_output_oneline(&files)?)
    } else if args.long {
        print!("{}", format_output_long(&files, &args)?);
    } else {
        print!("{}", format_output_short(&files)?);
    }

    // print directories
    for path in &directories {
        let paths = files_in(path, &args)?;
        if directories.len() > 1 {
            println!("\n{}:", file_name(path, false));
        }
        if args.oneline {
            print!("{}", format_output_oneline(&paths)?)
        } else if args.long {
            print!("{}", format_output_long(&paths, &args)?);
        } else {
            print!("{}", format_output_short(&paths)?);
        }
    }

    Ok(())
}
