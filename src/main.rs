use clap::{Arg, ArgAction, Command};
use flexi_logger::{detailed_format, Duplicate, FileSpec, Logger};
use log::{error, warn};
use owo_colors::colored::*;

use std::{
    fs, io,
    path::{Path, PathBuf},
    process,
};

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
        // let path = Path::new(arg).to_path_buf();

        // construct Config
        let config = Config::new(type_flag, show_errors_flag);

        // start
        if let Err(err) = get_metadata(config, path.to_path_buf()) {
            error!("ERROR reading path {}: {}", path.display(), err);
            process::exit(1);
        }
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

fn get_metadata(config: Config, path: PathBuf) -> io::Result<()> {
    if let Some(ext) = path.extension() {
        println!("{}", ext.to_string_lossy().to_string());
    }

    Ok(())
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
