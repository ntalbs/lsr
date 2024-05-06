use chrono::{DateTime, Local};
use clap::Parser;
use std::{
    fs::{self, Metadata},
    io,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};
use tabular::{Row, Table};
use users::{get_group_by_gid, get_user_by_uid};

#[derive(Debug, Default, Parser)]
#[clap(version, about = "A Very simple ls clone")]
pub struct Args {
    #[clap(default_value = ".")]
    paths: Vec<String>,
    #[clap(short('l'), long, default_value_t = false, help = "Long listing")]
    long: bool,
    #[clap(short('a'), long, default_value_t = false, help = "Show all files")]
    show_all: bool,
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

fn user_name(md: &Metadata) -> String {
    let uid = md.uid();
    get_user_by_uid(uid)
        .map(|u| u.name().to_string_lossy().into_owned())
        .unwrap_or_else(|| uid.to_string())
}

fn group_name(md: &Metadata) -> String {
    let gid = md.gid();
    get_group_by_gid(gid)
        .map(|g| g.name().to_string_lossy().into_owned())
        .unwrap_or_else(|| gid.to_string())
}

fn modified_date(md: &Metadata) -> String {
    let modified: DateTime<Local> = DateTime::from(md.modified().unwrap());
    modified.format("%Y-%m-%d %H:%M").to_string()
}

fn file_name(path: &Path) -> String {
    let mut name = path
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_else(|| "".to_string());
    if path.is_symlink() {
        if let Ok(target) = fs::read_link(path) {
            name.push_str(" -> ");
            name.push_str(&target.to_string_lossy());
        }
    } else if path.is_dir() {
        name.push('/');
    }
    name
}

fn format_output(paths: &[PathBuf]) -> io::Result<String> {
    let fmt = "{:<}{:<}  {:>}  {:<}  {:<}  {:>}  {:<}  {:<}";
    let mut table = Table::new(fmt);

    for path in paths {
        let md = path.metadata()?;
        table.add_row(
            Row::new()
                .with_cell(file_type(path))
                .with_cell(format_mode(md.mode()))
                .with_cell(md.nlink())
                .with_cell(user_name(&md))
                .with_cell(group_name(&md))
                .with_cell(md.len())
                .with_cell(modified_date(&md))
                .with_cell(file_name(path)),
        );
    }
    Ok(format!("{table}"))
}

fn find_files(paths: &[String], show_all: bool) -> io::Result<Vec<PathBuf>> {
    let mut results = vec![];
    for name in paths {
        match fs::metadata(name) {
            Ok(md) => {
                if md.is_dir() {
                    for entry in fs::read_dir(name)? {
                        let entry = entry?;
                        let path = entry.path();
                        let is_hidden = path.file_name().map_or(false, |file_name| {
                            file_name.to_string_lossy().starts_with('.')
                        });
                        if show_all || !is_hidden {
                            results.push(path);
                        }
                    }
                } else {
                    results.push(PathBuf::from(name));
                }
            }
            Err(e) => {
                eprintln!("{name}: {e}");
            }
        }
    }
    results.sort();

    Ok(results)
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let paths = find_files(&args.paths, args.show_all)?;

    if args.long {
        print!("{}", format_output(&paths)?);
    } else {
        for path in paths {
            println!("{}", path.file_name().unwrap().to_string_lossy());
        }
    }

    Ok(())
}
