mod logsetup;

use std::fs;

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use log::*;

/// Non-destructively upgrade Falcon BMS.cfg and friends
#[derive(Parser, Debug)]
struct Args {
    /// Verbosity (-v, -vv, -vvv, etc.)
    #[clap(short, long, parse(from_occurrences))]
    verbose: u8,

    #[clap(short, long, arg_enum, default_value = "auto")]
    color: logsetup::Color,

    /// The directory containing the previous BMS config
    #[clap(short, long)]
    old: Utf8PathBuf,

    /// The directory containing the new BMS config
    #[clap(short, long)]
    new: Utf8PathBuf,
}

type Config = rustc_hash::FxHashMap<String, String>;

struct Configurations {
    base: Config,
    user: Option<Config>,
}

fn parse_config(cfg: Vec<u8>, from: &Utf8Path) -> Result<Config> {
    let cfg: &str = std::str::from_utf8(&cfg)?;
    let lines = cfg.lines().filter_map(|line| {
        let line: &str = line.find("//").map(|i| &line[..i]).unwrap_or(line);
        if line.is_empty() {
            return None;
        }
        let tokens: Vec<_> = line.split_whitespace().collect();
        if tokens.len() != 3 || tokens[0] != "set" {
            warn!("{from}: Odd line '{line}'");
            return None;
        }
        Some((tokens[1], tokens[2]))
    });

    let mut parsed = Config::default();
    for (k, v) in lines {
        if parsed.insert(k.to_string(), v.to_string()).is_some() {
            warn!("{from}: Duplicate entries for '{k}'");
        }
    }
    Ok(parsed)
}

fn load_config_from_directory(dir: &Utf8Path) -> Result<Configurations> {
    let base_path = dir.join("Falcon BMS.cfg");

    let base = fs::read(&base_path)
        .or_else(|_| {
            debug!("Couldn't read {base_path}, trying lowercase");
            fs::read(dir.join("falcon bms.cfg"))
        })
        .with_context(|| format!("Couldn't read {base_path} (upper or lowercase)"))
        .and_then(|bytes| parse_config(bytes, &base_path))
        .with_context(|| format!("Couldn't parse {base_path}"))?;

    let user_path = dir.join("Falcon BMS User.cfg");
    let user = match fs::read(&user_path) {
        Ok(bytes) => Some(
            parse_config(bytes, &user_path)
                .with_context(|| format!("Couldn't parse {user_path})"))?,
        ),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => {
            warn!("Couldn't read user config: {}", e);
            None
        }
    };

    Ok(Configurations { base, user })
}

// We don't have to sort things, but stable output is nice.
// (We could also just make Config a sorted collection - a BTreeMap - from the start,
// but I'm betting hash lookups across a bunch of strings will save us some time.)
fn print_sorted(cfg: Config) {
    let mut sorted: Vec<(_, _)> = cfg.into_iter().collect();
    sorted.sort();
    for (k, v) in &sorted {
        println!("set {k} {v}");
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    logsetup::init_logger(args.verbose, args.color);

    let old_config = load_config_from_directory(&args.old)
        .with_context(|| format!("Couldn't load old config from {}", args.old))?;

    let new_config = load_config_from_directory(&args.new)
        .with_context(|| format!("Couldn't load new config from {}", args.new))?;

    let mut merged_user_config = old_config.user.unwrap_or_default();
    for (k, v) in new_config.user.unwrap_or_default() {
        if let Some(prev) = merged_user_config.insert(k.clone(), v.clone()) {
            if v != prev {
                trace!("In the user config, {k} changed from {prev} to {v}");
            }
        }
    }

    let mut changed = Config::default();
    let mut removed = Config::default();
    for (k, v) in old_config.base {
        if let Some(nv) = new_config.base.get(&k) {
            if v == *nv {
                continue;
            }
            trace!("In the base config, {k} changed from {v} to {nv}");
            assert!(changed.insert(k, v).is_none());
        } else {
            removed.insert(k, v);
        }
    }

    // Scrub anything set in user config from our base config sections.
    for k in merged_user_config.keys() {
        let mut scrubbed = changed.remove(k).is_some();
        scrubbed |= removed.remove(k).is_some();
        if scrubbed {
            trace!("{k} is in the user config, ignoring what's in base configs");
        }
    }

    if !merged_user_config.is_empty() {
        println!("// Merged user config:");
        print_sorted(merged_user_config);
    }

    if !removed.is_empty() {
        println!();
        println!("// These settings were in the old Falcon BMS.cfg but not the new.");
        println!("// They might be unused in the new version of BMS,");
        println!("// or maybe you set them! Clean up any you don't want.");
        print_sorted(removed)
    }

    if !changed.is_empty() {
        println!();
        println!("// These settings were changed from the old Falcon BMS.cfg to the new.");
        println!("// DELETE THEM IF YOU DIDN'T SET THEM YOURSELF!");
        println!("// (If you didn't, there's probably a reason they changed.)");
        print_sorted(changed);
    }

    Ok(())
}

fn main() {
    run().unwrap_or_else(|e| {
        error!("{:?}", e);
        std::process::exit(1);
    });
}
