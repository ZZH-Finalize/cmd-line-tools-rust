use clap::Parser;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_yml;
use std::{
    collections::HashSet,
    fs::{self, File},
    io::{BufRead, BufReader},
    net::IpAddr,
    path::{Path, PathBuf},
    process::ExitCode,
    str::FromStr,
};
use strum_macros;

#[derive(Debug, Parser)]
#[command(version = "0.0.1")]
#[command(about = "Generate YAML proxy file for Clash from plain text")]
struct Cli {
    /// Set verbose Level to display debug information
    #[arg(short, long = "verbose", default_value = "0")]
    verbose_level: u8,

    /// Set slice length
    #[arg(short, long, default_value = "0")]
    slice: usize,

    /// Set default proxy type
    #[arg(short = 't', long = "type", default_value = "socks5")]
    ptype: ProxyType,

    /// Source files or dirs that contains the proxy informations
    files_or_dirs: Vec<PathBuf>,
}

lazy_static! {
    static ref OPTIONS: Cli = Cli::parse();
}

#[derive(
    Debug,
    Hash,
    PartialEq,
    Eq,
    Clone,
    Serialize,
    Deserialize,
    strum_macros::Display,
    strum_macros::EnumString,
)]
enum ProxyType {
    #[strum(ascii_case_insensitive)]
    HTTP,

    #[strum(ascii_case_insensitive)]
    HTTPS,

    #[strum(ascii_case_insensitive)]
    Socks4,

    #[strum(ascii_case_insensitive)]
    Socks5,
}

#[derive(Debug, Hash, Serialize, Deserialize, PartialEq, Eq)]
struct Proxy {
    name: String,
    #[serde(rename = "server")]
    ip_addr: IpAddr,
    port: u32,
    #[serde(rename = "type")]
    ptype: ProxyType,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProxyGroups {
    name: String,
    #[serde(rename = "type")]
    gtype: String,
    proxies: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProxyConfig {
    proxies: Vec<Proxy>,
    #[serde(rename = "proxy-groups")]
    proxy_groups: Vec<ProxyGroups>,
}

fn get_proxies(file_name: &Path) -> HashSet<Proxy> {
    let mut proxies = HashSet::new();

    if file_name.is_dir() {
        return proxies;
    }

    let file = File::open(file_name).expect("file read error");
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();

        if OPTIONS.verbose_level > 2 {
            println!("{}", line);
        }

        let parts: Vec<_> = line.split(':').collect();
        let proxy_name = String::from_str(parts[0]).unwrap() + " - " + parts[1];

        match parts.len() {
            2 => {
                proxies.insert(Proxy {
                    name: proxy_name,
                    ip_addr: IpAddr::from_str(parts[0]).unwrap(),
                    port: u32::from_str(parts[1]).unwrap(),
                    ptype: OPTIONS.ptype.clone(),
                });
            }

            3 => {
                proxies.insert(Proxy {
                    name: proxy_name,
                    ip_addr: IpAddr::from_str(parts[0]).unwrap(),
                    port: u32::from_str(parts[1]).unwrap(),
                    ptype: ProxyType::from_str(parts[2]).unwrap(),
                });
            }

            _ => {
                println!("line format error");
            }
        }
    }

    proxies
}

fn generate_yaml(file_path: &PathBuf, proxies: HashSet<Proxy>) -> () {
    let mut proxy_groups = ProxyGroups {
        name: String::from(file_path.to_str().unwrap()),
        gtype: String::from("select"),
        proxies: Vec::new(),
    };

    for proxy in &proxies {
        proxy_groups.proxies.push(proxy.name.clone());
    }

    let yaml_data = ProxyConfig {
        proxies: Vec::from_iter(proxies),
        proxy_groups: vec![proxy_groups],
    };

    match serde_yml::to_string(&yaml_data) {
        Ok(yaml_string) => {

            let mut file_path = file_path.clone();
            file_path.set_extension("yaml");

            match fs::write(file_path, yaml_string) {
                Ok(..) => {}
                Err(e) => {
                    println!("error occurs when writing data to file: {}", e);
                }
            }
        }

        Err(e) => {
            println!("generate yaml string error: {}", e);
        }
    }
}

fn main() -> ExitCode {
    if 0 == OPTIONS.files_or_dirs.len() {
        return ExitCode::SUCCESS;
    }

    for fn_or_dirn in &OPTIONS.files_or_dirs {
        let proxies = get_proxies(&fn_or_dirn);
        // let file = File::open(fn_or_dirn).unwrap();
        // let yaml_data: ProxyConfig = serde_yml::from_reader(file).unwrap();

        // println!("{:#?}", yaml_data);

        generate_yaml(fn_or_dirn, proxies);
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {}
