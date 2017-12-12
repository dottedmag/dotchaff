// SPDX-License-Identifier: ISC
extern crate regex;
extern crate walkdir;
#[macro_use]
extern crate lazy_static;

use std::env;
use std::cmp::Ordering;
use std::fs::{File, read_dir};
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::io::{BufReader, BufRead};
use regex::{Regex, RegexSet};
use walkdir::{WalkDir, DirEntry};

#[derive(Debug)]
struct MatchSet {
    set: RegexSet,
    rxs: Vec<Regex>
}

type Config = HashMap<String, Vec<String>>;
type Matcher = HashMap<String, MatchSet>;

lazy_static! {
    static ref HEADER: Regex = Regex::new(r"^\[([a-zA-Z0-9-_]+)\]$").unwrap();
    static ref WS: Regex = Regex::new(r"^\s*(#.*)?$").unwrap();
}

fn read_config(path: &Path) -> Config {
    let mut config: Config = HashMap::new();
    let file = match File::open(&path) {
        Ok(file) => file,
        Err(e) => panic!("Failed to open configuration file {}: {}",
                         path.display(), e)
    };
    let mut current_target: Option<String> = None;
    for line in BufReader::new(file).lines() {
        let line = match line {
            Ok(line) => line,
            Err(e) => panic!("Unable to read from configuration file {}: {}",
                             path.display(), e)
        };
        if WS.is_match(&line) {
            continue;
        }
        if let Some(matched) = HEADER.captures(&line) {
            current_target = Some(matched[1].to_string());
            continue;
        }
        match current_target {
            Some(ref target) => {
                if !config.contains_key(target) {
                    config.insert(target.clone(), Vec::new());
                }
                config.get_mut(target).unwrap().push(line);
            },
            None => panic!("{}: line '{}' with no target", path.display(), line)
        };
    }
    config
}

fn merge_configs(configs: Vec<Config>) -> Config {
    let mut ret_config = Config::new();
    for config in configs {
        for (target, lines) in config {
            if !ret_config.contains_key(&target) {
                ret_config.insert(target.clone(), Vec::new());
            }
            for line in lines {
                ret_config.get_mut(&target).unwrap().push(line);
            }
        }
    }
    ret_config
}

fn prepare_matcher(config: Config) -> Matcher {
    let mut matcher = Matcher::new();
    for (target, lines) in config {
        let mut rxs = Vec::new();
        for line in &lines {
            let rx = match Regex::new(&line) {
                Ok(rx) => rx,
                Err(e) => panic!("Unable to parse regex {}: {}", line, e)
            };
            rxs.push(rx);
        }
        let rs = RegexSet::new(&lines).unwrap();
        matcher.insert(target.clone(), MatchSet { set: rs, rxs: rxs });
    }
    matcher
}

fn fn_cmp(a: &DirEntry, b: &DirEntry) -> Ordering {
    a.file_name().cmp(b.file_name())
}

fn match_len(path: &str, ms: &MatchSet) -> Option<usize> {
    let mut longest = 0;
    for rx in &ms.rxs {
        if let Some(m) = rx.find(path) {
            if longest < m.end() {
                longest = m.end();
            }
        }
    }
    if longest > 0 { Some(longest) } else { None }
}

fn do_match2(path: &str, matcher: &Matcher) -> Result<String, HashSet<String>> {
    let mut longest = 0;
    let mut longest_targets: HashSet<String> = HashSet::new();
    for (tgt, ms) in matcher {
        if let Some(len) = match_len(path, &ms) {
            if len > longest {
                longest = len;
                longest_targets = HashSet::new();
            }
            if len == longest {
                longest_targets.insert(tgt.clone());
            }
        }
    }
    if longest_targets.len() == 1 {
        Ok(longest_targets.iter().next().unwrap().clone())
    } else {
        Err(longest_targets)
    }
}

// Fast path: if 0..1 regex sets match, return that. Otherwise delegate to slow
// do_match2()
fn do_match(path: &str, matcher: &Matcher) -> Result<String, HashSet<String>> {
    let mut target: Option<String> = None;
    for (tgt, ms) in matcher {
        if ms.set.is_match(path) {
            if target == None {
                target = Some(tgt.clone());
            } else {
                return do_match2(path, matcher);
            }
        }
    }
    match target {
        Some(target) => Ok(target),
        None => Err(HashSet::new())
    }
}

fn main()
{
    let home_dir = match env::home_dir() {
        Some(path) => path,
        None => panic!("Unable to determine $HOME")
    };
    let mut config_dir: PathBuf = home_dir.clone();
    config_dir.push(".config/dotchaff");
    let rd = match read_dir(&config_dir) {
        Ok(dir) => dir,
        Err(e) => panic!("Unable to open config directory {}: {}",
                         config_dir.display(), e)
    };
    let mut configs: Vec<Config> = Vec::new();
    for entry in rd {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => panic!("Failed to read directory {}: {}",
                             config_dir.display(), e)
        };
        let path = entry.path();
        let filetype = match entry.file_type() {
            Ok(filetype) => filetype,
            Err(e) => panic!("Failed to read file type of {}: {}",
                             path.display(), e)
        };
        if !filetype.is_file() {
            continue;
        }
        configs.push(read_config(&path));
    }
    let matcher = prepare_matcher(merge_configs(configs));
    for entry in WalkDir::new(&home_dir).sort_by(fn_cmp) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => panic!("Failed to walk directory {}: {}",
                             home_dir.display(), e)
        };
        if entry.file_type().is_dir() {
            continue;
        }
        let path = match entry.path().strip_prefix(&home_dir) {
            Ok(path) => path,
            Err(e) => panic!("Failed to strip prefix {} from path {}: {}",
                             &home_dir.display(), entry.path().display(), e)
        };
        match do_match(&path.to_string_lossy(), &matcher) {
            Ok(_path) => (),
            Err(hs) => println!("!! {}: {:?}", path.display(), hs)
        }
    }
}
