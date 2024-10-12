mod cli;

use crate::cli::{Args, TimeStyle};
use chrono::{DateTime, Local};
use clap::Parser;
use colored::{ColoredString, Colorize};
use core::str;
use std::{
    fs::{self, FileType, Metadata},
    io::{self, Error},
    os::unix::{
        ffi::OsStrExt,
        fs::{FileTypeExt, MetadataExt},
    },
    path::{Path, PathBuf},
};
use tabular::{Row, Table};
use term_grid::{Direction, Filling, Grid, GridOptions};
use terminal_size::{terminal_size, Width};
use uzers::{get_group_by_gid, get_user_by_uid};

fn file_type(file_type: FileType) -> ColoredString {
    if file_type.is_symlink() {
        "l".cyan()
    } else if file_type.is_dir() {
        "d".blue()
    } else if file_type.is_file() {
        "-".white()
    } else if file_type.is_block_device() {
        "b".yellow()
    } else if file_type.is_char_device() {
        "c".magenta()
    } else if file_type.is_fifo() {
        "p".blue()
    } else if file_type.is_socket() {
        "s".green()
    } else {
        "?".red()
    }
}

#[rustfmt::skip]
fn format_mode(md: &Metadata, has_xattr: bool) -> String {
    let mode = md.mode();
    format!("{}{}{}{}{}{}{}{}{}{}{}",
        file_type(md.file_type()),
        if mode & 0b100000000 != 0 { "r".yellow() } else { "-".white() },
        if mode & 0b010000000 != 0 { "w".red()    } else { "-".white() },
        if mode & 0b001000000 != 0 { "x".green()  } else { "-".white() },
        if mode & 0b000100000 != 0 { "r".yellow() } else { "-".white() },
        if mode & 0b000010000 != 0 { "w".red()    } else { "-".white() },
        if mode & 0b000001000 != 0 { "x".green()  } else { "-".white() },
        if mode & 0b000000100 != 0 { "r".yellow() } else { "-".white() },
        if mode & 0b000000010 != 0 { "w".red()    } else { "-".white() },
        if mode & 0b000000001 != 0 { "x".green()  } else { "-".white() },
        if has_xattr { "@" } else { "" }
    )
}

fn user_name(uid: u32) -> ColoredString {
    get_user_by_uid(uid)
        .map(|u| u.name().to_string_lossy().to_string())
        .unwrap_or_else(|| uid.to_string())
        .yellow()
}

fn group_name(gid: u32) -> ColoredString {
    get_group_by_gid(gid)
        .map(|g| g.name().to_string_lossy().to_string())
        .unwrap_or_else(|| gid.to_string())
        .yellow()
}

fn modified_date(md: &Metadata, time_style: TimeStyle) -> String {
    let modified: DateTime<Local> = DateTime::from(md.modified().unwrap());
    match time_style {
        TimeStyle::Default => date_default(modified),
        TimeStyle::Iso => date_iso(modified),
        TimeStyle::Relative => date_relative(modified),
    }
}

fn date_default(date_time: DateTime<Local>) -> String {
    let now = Local::now();
    let duration = now - date_time;

    if duration.num_days() / 365 > 1 {
        format!("{}", date_time.format("%e %b  %Y").to_string().magenta())
    } else {
        format!("{}", date_time.format("%e %b %H:%M").to_string().magenta())
    }
}

fn date_iso(date_time: DateTime<Local>) -> String {
    format!(
        "{} {}",
        date_time.format("%Y-%m-%d").to_string().magenta(),
        date_time.format("%H:%M").to_string().bright_magenta()
    )
}

fn date_relative(date_time: DateTime<Local>) -> String {
    fn pluralize(n: i64, s: &str) -> String {
        let unit = if n == 1 {
            s.to_string()
        } else {
            format!("{}s", s)
        };
        format!("{:>2} {}", n.to_string().bright_magenta(), unit.magenta())
    }

    let now = Local::now();
    let duration = now - date_time;
    if duration.num_minutes() < 1 {
        pluralize(duration.num_seconds(), "second")
    } else if duration.num_hours() < 1 {
        pluralize(duration.num_minutes(), "minute")
    } else if duration.num_days() < 1 {
        pluralize(duration.num_hours(), "hour")
    } else if duration.num_days() < 30 {
        pluralize(duration.num_days(), "day")
    } else if duration.num_days() < 365 {
        pluralize(duration.num_days() / 30, "month")
    } else {
        pluralize(duration.num_days() / 365, "year")
    }
}

fn file_name(path: &Path, long: bool) -> String {
    if path == PathBuf::from(".") {
        return format!("{}{}", ".".blue(), "/".white());
    } else if path == PathBuf::from("..") {
        return format!("{}{}", "..".blue(), "/".white());
    }

    let name = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();
    if path.is_symlink() {
        if long {
            if let Ok(target) = fs::read_link(path) {
                if target.exists() {
                    return format!(
                        "{}{}{}",
                        name.cyan(),
                        " -> ".cyan(),
                        &target.to_string_lossy().cyan()
                    );
                } else {
                    return format!(
                        "{}{}{}",
                        name.cyan(),
                        " -> ".red(),
                        &target.to_string_lossy().red()
                    );
                }
            }
        } else {
            return format!("{}{}", name.cyan(), "@".white());
        }
    } else if path.is_dir() {
        return format!("{}{}", name.blue(), "/".white());
    }
    format!("{}", name.white())
}

fn file_size(md: &Metadata, bytes: bool) -> ColoredString {
    if !md.is_file() {
        return "-".white();
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
    .green()
}

fn format_output_oneline(paths: &[PathBuf]) -> io::Result<String> {
    let mut output = String::new();
    for p in paths {
        output.push_str(&file_name(p, true));
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

#[rustfmt::skip]
fn format_output_long(paths: &[PathBuf], args: &Args) -> io::Result<String> {
    let fmt = "{:>} {:<} {:>} {:<} {:<} {:>} {:<} {:<}";
    let mut table = Table::new(fmt);

    for path in paths {
        let md = if path.is_symlink() {
            path.symlink_metadata()?
        } else {
            path.metadata()?
        };

        let mut xattrs = xattr::list(path)?
            .map(|attr| attr.to_string_lossy().to_string())
            .peekable();

        table.add_row(
            Row::new()
                .with_ansi_cell(if args.inode { md.ino().to_string().cyan() } else { "".white() })
                .with_ansi_cell(if args.no_permissions { "".to_string() } else { format_mode(&md, xattrs.peek().is_some()) })
                .with_ansi_cell(if args.links { md.nlink().to_string() } else { "".to_string() })
                .with_ansi_cell(user_name(md.uid()))
                .with_ansi_cell(if args.group { group_name(md.gid()) } else { "".white() })
                .with_ansi_cell(file_size(&md, args.bytes))
                .with_ansi_cell(modified_date(&md, args.time_style))
                .with_ansi_cell(file_name(path, true))
        );
        if args.extended {
            while let Some(attr) = xattrs.next() {
                table.add_row(
                    Row::new()
                        .with_ansi_cell("")
                        .with_ansi_cell("")
                        .with_ansi_cell("")
                        .with_ansi_cell("")
                        .with_ansi_cell("")
                        .with_ansi_cell("")
                        .with_ansi_cell("")
                        .with_ansi_cell(
                            if xattrs.peek().is_none() { format!("└── {attr}") } else { format!("├── {attr}") }
                        ),
                );
            }
        }
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
