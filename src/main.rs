#[macro_use]
extern crate prettytable;
use prettytable::{format, Table};

use clap::{Arg, ArgAction, Command};
use flexi_logger::{detailed_format, Duplicate, FileSpec, Logger};
use log::error;
use owo_colors::colored::*;

use std::{
    fs, io,
    path::{Path, PathBuf},
    process,
    time::SystemTime,
};

const KB: u64 = 1024;
const MB: u64 = 1024_u64.pow(2);
const GB: u64 = 1024_u64.pow(3);
const TB: u64 = 1024_u64.pow(4);

// red
const EXECUTABLE: &[&'static str] = &["exe", "msi", "bat"];
// yellow
const SPECIAL: &[&'static str] = &[
    "md", "cgf", "conf", "config", "ini", "json", "tml", "toml", "yaml", "yml", "csv", "markdown",
    "org", "rst", "xml",
];
// green
const PROGRAMMING: &[&'static str] = &[
    "py", "pl", "rs", "c", "cpp", "awk", "vb", "cabal", "clj", "cs", "csx", "css", "h", "hpp",
    "dart", "ex", "exs", "elc", "elm", "erl", "fs", "go", "hs", "ipynb", "java", "bsh", "js", "jl",
    "kt", "tex", "lisp", "lua", "matlab", "pas", "p", "php", "ps1", "r", "rb", "scala", "sh",
    "bash", "zsh", "fish", "sql", "swift", "ts", "tsx", "vim", "cmake", "make",
];
// pink
const MEDIA: &[&'static str] = &[
    "bmp", "gif", "jpeg", "jpg", "png", "svg", "avi", "mp4", "wmv", "wma", "mp3", "wav", "mid",
    "ttf",
];
// red
const OFFICE: &[&'static str] = &[
    "doc", "docx", "epub", "odt", "pdf", "ps", "xls", "xlsx", "ods", "xlr", "ppt", "pptx", "odp",
    "pps", "ics",
];
// cyan
const ARCHIVES: &[&'static str] = &[
    "apk", "deb", "rpm", "xbps", "bag", "bin", "dmg", "img", "iso", "toast", "vcd", "7z", "arj",
    "gz", "zip", "pkg", "tar", "jar", "rar", "tgz", "z", "zst", "xz", "tgz",
];
// darkgray
const OTHER: &[&'static str] = &["~", "git", "gitignore", "tmp", "lock", "txt"];

// TODO add size, modified, owner, group, permissions
struct Config {
    type_flag: bool,
    show_errors_flag: bool,
}

impl Config {
    fn new(type_flag: bool, show_errors_flag: bool) -> Self {
        Self {
            type_flag,
            show_errors_flag,
        }
    }
}

fn main() {
    // handle Ctrl+C
    ctrlc::set_handler(move || {
        println!(
            "{} {} {} {}",
            "Received Ctrl-C!".bold().red(),
            "🤬",
            "Exit program!".bold().red(),
            "☠",
        );
        process::exit(0)
    })
    .expect("Error setting Ctrl-C handler");

    // get config dir
    let config_dir = check_create_config_dir().unwrap_or_else(|err| {
        error!("Unable to find or create a config directory: {err}");
        process::exit(1);
    });

    // initialize the logger
    let _logger = Logger::try_with_str("info") // log warn and error
        .unwrap()
        .format_for_files(detailed_format) // use timestamp for every log
        .log_to_file(
            FileSpec::default()
                .directory(&config_dir)
                .suppress_timestamp(),
        ) // change directory for logs, no timestamps in the filename
        .append() // use only one logfile
        .duplicate_to_stderr(Duplicate::Info) // print infos, warnings and errors also to the console
        .start()
        .unwrap();

    // handle arguments
    let matches = witchfile().get_matches();
    let mut type_flag = matches.get_flag("type");
    let mut show_errors_flag = matches.get_flag("show-errors");
    let override_flag = matches.get_flag("override");

    // if override flag is set -> reset everything to default values
    if override_flag {
        type_flag = false;
        show_errors_flag = false;
    }

    if let Some(arg) = matches.get_one::<String>("arg") {
        // get search path from arguments
        let path = Path::new(arg);
        // FIXME remove?
        // let path = Path::new(arg).to_path_buf();

        // construct Config
        let config = Config::new(type_flag, show_errors_flag);

        // start
        get_metadata(config, path.to_path_buf());
    } else {
        // handle commands
        match matches.subcommand() {
            Some(("log", _)) => {
                if let Ok(logs) = show_log_file(&config_dir) {
                    println!("{}", "Available logs:".bold().yellow());
                    println!("{}", logs);
                } else {
                    error!("Unable to read logs");
                    process::exit(1);
                }
            }
            _ => {
                unreachable!();
            }
        }
    }
}

// build cli
fn witchfile() -> Command {
    Command::new("witchfile")
        .bin_name("wf")
        .before_help(format!(
            "{}\n{}",
            "WITCHFILE".bold().truecolor(250, 0, 104),
            "Leann Phydon <leann.phydon@gmail.com>".italic().dimmed()
        ))
        .about("WitchFile")
        .before_long_help(format!(
            "{}\n{}",
            "WICTHFILE".bold().truecolor(250, 0, 104),
            "Leann Phydon <leann.phydon@gmail.com>".italic().dimmed()
        ))
        .long_about(format!(
            "{}\n  {}\n",
            "WitchFile",
            "Get file metadata".truecolor(250, 0, 104)
        ))
        // TODO update version
        .version("1.0.0")
        .author("Leann Phydon <leann.phydon@gmail.com>")
        .arg_required_else_help(true)
        .arg(
            Arg::new("arg")
                .help("Add a path")
                .action(ArgAction::Set)
                .num_args(1)
                .value_names(["PATH"]),
        )
        .arg(
            Arg::new("type")
                .short('f')
                .long("type")
                .help("Show the filetype")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("override")
                .short('o')
                .long("override")
                .help("Override all previously set flags")
                .long_help(format!(
                    "{}\n{}\n{}",
                    "Override all previously set flags",
                    "This can be used when a custom alias for this command is set together with regularly used flags",
                    "This flag allows to disable these flags and specify new ones"
                ))
                // TODO if new args -> add here to this list to override if needed
                .overrides_with_all(["type", "show-errors"])
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("show-errors")
                .long("show-errors")
                .help("Show possible filesystem errors")
                .long_help(format!(
                    "{}\n{}",
                    "Show possible filesystem errors",
                    "For example for situations such as insufficient permissions",
                ))
                .action(ArgAction::SetTrue)
        )
        .subcommand(
            Command::new("log")
                .short_flag('L')
                .long_flag("log")
                .about("Show content of the log file"),
        )
}

fn get_metadata(config: Config, path: PathBuf) {
    if path.exists() {
        // Initiate table
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        // get filename
        let name = if let Some(name) = path.file_stem() {
            name.to_string_lossy().bold().to_string()
        } else {
            "-".dimmed().to_string()
        };
        table.set_titles(row!["Name".dimmed(), c->name]);

        // get filetype
        let filetype = if path.is_file() {
            "file".truecolor(180, 190, 130).to_string()
        } else if path.is_dir() {
            "directory".truecolor(180, 190, 130).to_string()
        } else if path.is_symlink() {
            "symlink".truecolor(180, 190, 130).to_string()
        } else {
            "-".dimmed().to_string()
        };
        table.add_row(row!["Type".dimmed(), r->filetype]);

        // get file extension
        let ext = if let Some(ext) = path.extension() {
            ext.to_string_lossy().truecolor(226, 120, 120).to_string()
        } else {
            "-".dimmed().to_string()
        };
        table.add_row(row!["Extension".dimmed(), r->ext]);

        // get file category
        let mut category = String::new();

        // FIXME
        // comparing extension to category doesn`t work with "=="
        // wrong results with "contains()"
        // -> e.g. "pdf" is matched to "programming" category
        if EXECUTABLE.iter().any(|it| ext.contains(it)) {
            let cstr = format!("{}", "executable".bold().truecolor(226, 120, 120));
            category.push_str(&cstr);
        } else if SPECIAL.iter().any(|it| ext.contains(it)) {
            let cstr = format!("{}", "special".truecolor(226, 164, 120));
            category.push_str(&cstr);
        } else if PROGRAMMING.iter().any(|it| ext.contains(it)) {
            let cstr = format!("{}", "programming".truecolor(180, 190, 130));
            category.push_str(&cstr);
        } else if OFFICE.iter().any(|it| ext.contains(it)) {
            let cstr = format!("{}", "office".truecolor(226, 120, 120));
            category.push_str(&cstr);
        } else if MEDIA.iter().any(|it| ext.contains(it)) {
            let cstr = format!("{}", "media".truecolor(173, 160, 211));
            category.push_str(&cstr);
        } else if ARCHIVES.iter().any(|it| ext.contains(it)) {
            let cstr = format!("{}", "archives".truecolor(137, 184, 194));
            category.push_str(&cstr);
        } else if OTHER.iter().any(|it| ext.contains(it)) {
            let cstr = format!("{}", "other".truecolor(107, 112, 137));
            category.push_str(&cstr);
        } else {
            let cstr = format!("{}", "-".dimmed());
            category.push_str(&cstr);
        }

        table.add_row(row!["Category".dimmed(), r->category]);

        // get file metadata
        let meta: Option<fs::Metadata> = if let Ok(m) = fs::metadata(path) {
            Some(m)
        } else {
            None
        };
        if let Some(meta) = meta {
            // get filesize
            let mut filesize = get_filesize(meta.clone());

            let mut fsize_unit = String::new();
            if let Some(f) = filesize.pop() {
                fsize_unit.push_str(&f.truecolor(50, 170, 130).to_string())
            } else {
                fsize_unit.push_str(&"".truecolor(198, 200, 209).to_string())
            }

            let mut fsize = String::new();
            if let Some(f) = filesize.pop() {
                fsize.push_str(&f.truecolor(102, 255, 179).to_string())
            } else {
                fsize.push_str(&"".truecolor(198, 200, 209).to_string())
            }

            let mut size = String::new();
            size.push_str(&fsize);
            size.push_str(&fsize_unit);
            table.add_row(row!["Size".dimmed(), r->size]);

            // get creation time
            match &meta.created() {
                Ok(time) => {
                    let humanreadable_time = to_humanreadable(time.to_owned());
                    table.add_row(row![
                        "Created".dimmed(),
                        r->humanreadable_time.truecolor(226, 164, 120)
                    ]);
                }
                _ => {
                    table.add_row(row!["Created".dimmed(), "-".dimmed()]);
                }
            }
            // get last access time
            match &meta.accessed() {
                Ok(time) => {
                    let humanreadable_time = to_humanreadable(time.to_owned());
                    table.add_row(row![
                        "Accessed".dimmed(),
                        r->humanreadable_time.truecolor(226, 164, 120)
                    ]);
                }
                _ => {
                    table.add_row(row!["Accessed".dimmed(), "-".dimmed()]);
                }
            }
            // get last modification time
            match &meta.modified() {
                Ok(time) => {
                    let humanreadable_time = to_humanreadable(time.to_owned());
                    table.add_row(row![
                        "Modified".dimmed(),
                        r->humanreadable_time.truecolor(226, 164, 120)
                    ]);
                }
                _ => {
                    table.add_row(row!["Modified".dimmed(), "-".dimmed()]);
                }
            }

            // get permissions
            if meta.permissions().readonly() {
                table.add_row(row![
                    "Restrictions".dimmed(),
                    r->"readonly".truecolor(250, 0, 104).dimmed()
                ]);
            } else {
                table.add_row(row!["Restrictions".dimmed(), r->"-".dimmed()]);
            }
        }

        // print table
        table.printstd();
    } else {
        error!("File \'{}\' not found", path.display());

        // error!("Error while reading \'{}\'", path.display());
        // match err.kind() {
        //     io::ErrorKind::NotFound => {
        //         error!("File not found: {err}");
        //     }
        //     io::ErrorKind::PermissionDenied => {
        //         error!("You don`t have access to the file: {err}");
        //     }
        //     io::ErrorKind::InvalidData => {
        //         error!("Found invalid data: {err}");
        //     }
        //     _ => {
        //         error!("An unexpected error occured: {}", err);
        //     }
        // }
    }
}

fn get_filesize(metadata: fs::Metadata) -> Vec<String> {
    // Convert filesize into human readable format
    let filesize = metadata.len();
    let mut fsize: Vec<String> = Vec::new();
    if filesize <= 0 {
        fsize.push("-".to_string());
    } else {
        match filesize {
            s if s > TB => {
                let size = ((filesize as f64 / TB as f64) * 10.0).round() / 10.0;
                fsize.push(size.to_string());
                fsize.push("T".to_string());
            }
            s if s > GB && s < TB => {
                let size = ((filesize as f64 / GB as f64) * 10.0).round() / 10.0;
                fsize.push(size.to_string());
                fsize.push("G".to_string());
            }
            s if s > MB && s < GB => {
                let size = ((filesize as f64 / MB as f64) * 10.0).round() / 10.0;
                fsize.push(size.to_string());
                fsize.push("M".to_string());
            }
            s if s > KB && s < MB => {
                let size = ((filesize as f64 / KB as f64) * 10.0).round() / 10.0;
                fsize.push(size.to_string());
                fsize.push("K".to_string());
            }
            s if s < KB => {
                fsize.push(filesize.to_string());
                fsize.push("B".to_string());
            }
            _ => {
                fsize.push("-".to_string());
            }
        }
    }

    fsize
}

fn to_humanreadable(systime: SystemTime) -> String {
    // Convert system time into human readable format
    let systemtime: u64 = SystemTime::now()
        .duration_since(systime)
        .unwrap_or_else(|err| {
            error!("Unable to get duration since the system is running: {err}");
            process::exit(1);
        })
        .as_secs();

    let mut human_readable = String::new();
    match systemtime {
        0..=59 => {
            human_readable.push_str(systemtime.to_string().as_str());
            human_readable.push_str(" secs ago");
        }
        60..=3599 => {
            let minutes = ((systemtime as f64 / 60.0) as f64).round();
            human_readable.push_str(minutes.to_string().as_str());
            human_readable.push_str(" mins ago");
        }
        3600..=86399 => {
            let hours = ((systemtime as f64 / 3600.0) as f64).round();
            human_readable.push_str(hours.to_string().as_str());
            human_readable.push_str("  hrs ago");
        }
        86400.. => {
            let days = ((systemtime as f64 / 86400.0) as f64).round();
            human_readable.push_str(days.to_string().as_str());
            human_readable.push_str(" days ago");
        }
    }

    human_readable
}

fn check_create_config_dir() -> io::Result<PathBuf> {
    let mut new_dir = PathBuf::new();
    match dirs::config_dir() {
        Some(config_dir) => {
            new_dir.push(config_dir);
            new_dir.push("sf");
            if !new_dir.as_path().exists() {
                fs::create_dir(&new_dir)?;
            }
        }
        None => {
            error!("Unable to find config directory");
        }
    }

    Ok(new_dir)
}

fn show_log_file(config_dir: &PathBuf) -> io::Result<String> {
    let log_path = Path::new(&config_dir).join("sf.log");
    match log_path.try_exists()? {
        true => {
            return Ok(format!(
                "{} {}\n{}",
                "Log location:".italic().dimmed(),
                &log_path.display(),
                fs::read_to_string(&log_path)?
            ));
        }
        false => {
            return Ok(format!(
                "{} {}",
                "No log file found:"
                    .truecolor(250, 0, 104)
                    .bold()
                    .to_string(),
                log_path.display()
            ))
        }
    }
}
