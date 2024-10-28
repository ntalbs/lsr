mod cli;
mod fs;

use crate::cli::Args;
use crate::fs::*;
use clap::Parser;
use colored::Colorize;
use std::{
    io::{self, Error},
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
    path::{Path, PathBuf},
};
use tabular::{Row, Table};
use term_grid::{Direction, Filling, Grid, GridOptions};
use terminal_size::{terminal_size, Width};

fn format_output_oneline(paths: &[PathBuf]) -> io::Result<String> {
    let mut output = String::new();
    for p in paths {
        output.push_str(&file_name(p, true));
        output.push('\n');
    }
    Ok(output)
}

fn format_output_short(paths: &[PathBuf], across: bool) -> io::Result<String> {
    let term_size = terminal_size();
    if let Some((Width(w), _)) = term_size {
        let cells = paths.iter().map(|p| file_name(p, false)).collect();
        let grid = Grid::new(
            cells,
            GridOptions {
                filling: Filling::Spaces(2),
                direction: if across {
                    Direction::LeftToRight
                } else {
                    Direction::TopToBottom
                },
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
        let md = metadata(path)?;
        let xattrs = xattrs(path);
        let mut xattrs = xattrs.iter().peekable();

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
    for entry in std::fs::read_dir(path)? {
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
        if args.all > 0 || !is_hidden {
            results.push(path);
        }
    }

    results.sort();

    if args.all >= 2 {
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
        print!("{}", format_output_short(&files, args.across)?);
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
            print!("{}", format_output_short(&paths, args.across)?);
        }
    }

    Ok(())
}
