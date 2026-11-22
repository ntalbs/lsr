use std::{
    fs::{self, FileType, Metadata}, io, os::unix::fs::{FileTypeExt, MetadataExt}, path::Path, time::SystemTime,
};

use colored::{ColoredString, Colorize};
use time::{OffsetDateTime, UtcOffset, macros::format_description};
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

fn format_date(date_time: OffsetDateTime, time_style: TimeStyle) -> String {
    match time_style {
        TimeStyle::Default => date_default(date_time),
        TimeStyle::Iso => date_iso(date_time),
        TimeStyle::Relative => date_relative(date_time),
    }
}

fn format_timestamp(ts: io::Result<SystemTime>, time_style: TimeStyle) -> String {
    ts.map(|ts| format_date(OffsetDateTime::from(ts), time_style))
        .unwrap_or_else(|_| "-".to_string())
}

pub(crate) fn modified_date(md: &Metadata, time_style: TimeStyle) -> String {
    format_timestamp(md.modified(), time_style)
}

pub(crate) fn accessed_date(md: &Metadata, time_style: TimeStyle) -> String {
    format_timestamp(md.accessed(), time_style)
}

pub(crate) fn created_date(md: &Metadata, time_style: TimeStyle) -> String {
    format_timestamp(md.created(), time_style)
}

pub(crate) fn changed_date(md: &Metadata, time_style: TimeStyle) -> String {
    let secs = md.ctime();
    let utc = OffsetDateTime::from_unix_timestamp(secs).unwrap_or(OffsetDateTime::UNIX_EPOCH);
    let offset = UtcOffset::local_offset_at(utc).unwrap_or(UtcOffset::UTC);
    let dt = utc.to_offset(offset);
    format_date(dt, time_style)
}

pub(crate) fn date_default(date_time: OffsetDateTime) -> String {
    let now = OffsetDateTime::now_local().unwrap();
    let duration = now - date_time;

    if duration.whole_days() / 365 > 1 {
        let fmt = format_description!("[day padding:space] [month repr:short] [year]");
        let formatted = date_time.format(&fmt).unwrap_or_default();
        format!("{}", formatted.magenta())
    } else {
        let fmt = format_description!("[day padding:space] [month repr:short] [hour]:[minute]");
        let formatted = date_time.format(&fmt).unwrap_or_default();
        format!("{}", formatted.magenta())
    }
}

pub(crate) fn date_iso(date_time: OffsetDateTime) -> String {
    let iso = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    date_time.format(&iso).unwrap_or_default()
}

pub(crate) fn date_relative(date_time: OffsetDateTime) -> String {
    fn pluralize(n: i64, s: &str) -> String {
        let unit = if n == 1 {
            s.to_string()
        } else {
            format!("{s}s")
        };
        format!("{:>2} {}", n.to_string().bright_magenta(), unit.magenta())
    }

    let now = OffsetDateTime::now_local().unwrap();
    let duration = now - date_time;
    if duration.whole_minutes() < 1 {
        pluralize(duration.whole_seconds(), "second")
    } else if duration.whole_hours() < 1 {
        pluralize(duration.whole_minutes(), "minute")
    } else if duration.whole_days() < 1 {
        pluralize(duration.whole_hours(), "hour")
    } else if duration.whole_days() < 30 {
        pluralize(duration.whole_days(), "day")
    } else if duration.whole_days() < 365 {
        pluralize(duration.whole_days() / 30, "month")
    } else {
        pluralize(duration.whole_days() / 365, "year")
    }
}

pub(crate) fn file_name(path: &Path, long: bool) -> String {
    if path == "." {
        return format!("{}/", ".".blue());
    } else if path == ".." {
        return format!("{}/", "..".blue());
    }

    let name = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    let md = metadata(path).unwrap();
    let file_type = md.file_type();
    let is_executable = md.mode() & 0b001001001 != 0;

    if file_type.is_symlink() {
        if long {
            if let Ok(target) = fs::read_link(path) {
                if target.exists() {
                    return format!(
                        "{}{}{}",
                        name.cyan(),
                        " -> ".cyan(),
                        file_name(&target, false)
                    );
                } else {
                    return format!(
                        "{}{}{}",
                        name.cyan(),
                        " -> ".red(),
                        target.to_string_lossy().red()
                    );
                }
            }
        } else {
            return format!("{}@", name.cyan());
        }
    } else if file_type.is_dir() {
        return format!("{}/", name.blue());
    } else if file_type.is_fifo() {
        return format!("{}|", name.yellow());
    } else if file_type.is_socket() {
        return format!("{}=", name.red());
    } else if is_executable {
        return format!("{}*", name.green());
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
