use std::{
    fs::{self, FileType, Metadata},
    io,
    os::unix::fs::{FileTypeExt, MetadataExt},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Local};
use colored::{ColoredString, Colorize};
use uzers::{get_group_by_gid, get_user_by_uid};

use crate::cli::TimeStyle;

pub(crate) fn file_type(file_type: FileType) -> ColoredString {
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

pub(crate) fn metadata(path: &Path) -> io::Result<fs::Metadata> {
    if path.is_symlink() {
        path.symlink_metadata()
    } else {
        path.metadata()
    }
}

#[rustfmt::skip]
pub(crate) fn format_mode(md: &Metadata, has_xattr: bool) -> String {
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

pub(crate) fn xattrs(path: &Path) -> Vec<String> {
    match xattr::list(path) {
        Ok(attrs) => attrs
            .map(|attr| attr.to_string_lossy().to_string())
            .collect(),
        Err(_) => vec![],
    }
}

pub(crate) fn user_name(uid: u32) -> ColoredString {
    get_user_by_uid(uid)
        .map(|u| u.name().to_string_lossy().to_string())
        .unwrap_or_else(|| uid.to_string())
        .bright_yellow()
}

pub(crate) fn group_name(gid: u32) -> ColoredString {
    get_group_by_gid(gid)
        .map(|g| g.name().to_string_lossy().to_string())
        .unwrap_or_else(|| gid.to_string())
        .yellow()
}

pub(crate) fn modified_date(md: &Metadata, time_style: TimeStyle) -> String {
    let modified: DateTime<Local> = DateTime::from(md.modified().unwrap());
    match time_style {
        TimeStyle::Default => date_default(modified),
        TimeStyle::Iso => date_iso(modified),
        TimeStyle::Relative => date_relative(modified),
    }
}

pub(crate) fn date_default(date_time: DateTime<Local>) -> String {
    let now = Local::now();
    let duration = now - date_time;

    if duration.num_days() / 365 > 1 {
        format!("{}", date_time.format("%e %b  %Y").to_string().magenta())
    } else {
        format!("{}", date_time.format("%e %b %H:%M").to_string().magenta())
    }
}

pub(crate) fn date_iso(date_time: DateTime<Local>) -> String {
    format!(
        "{} {}",
        date_time.format("%Y-%m-%d").to_string().magenta(),
        date_time.format("%H:%M").to_string().bright_magenta()
    )
}

pub(crate) fn date_relative(date_time: DateTime<Local>) -> String {
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

pub(crate) fn file_name(path: &Path, long: bool) -> String {
    if path == PathBuf::from(".") {
        return format!("{}/", ".".blue());
    } else if path == PathBuf::from("..") {
        return format!("{}/", "..".blue());
    }

    let name = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();
    let file_type = metadata(path).unwrap().file_type();

    if file_type.is_symlink() {
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
            return format!("{}@", name.cyan());
        }
    } else if file_type.is_dir() {
        return format!("{}/", name.blue());
    }
    name
}

pub(crate) fn file_size(md: &Metadata, bytes: bool) -> ColoredString {
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
