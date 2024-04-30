use clap::Parser;
use lazy_static::lazy_static;
use serde::Serialize;
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
    Copy,
    Serialize,
    strum_macros::Display,
    strum_macros::EnumString,
)]
enum ProxyType {
    #[strum(ascii_case_insensitive)]
    #[serde(rename = "http")]
    HTTP,

    #[strum(ascii_case_insensitive)]
    #[serde(rename = "https")]
    HTTPS,

    #[strum(ascii_case_insensitive)]
    #[serde(rename = "socks4")]
    Socks4,

    #[strum(ascii_case_insensitive)]
    #[serde(rename = "socks5")]
    Socks5,
}

#[derive(Debug, Hash, Serialize, PartialEq, Eq)]
struct Proxy {
    name: String,
    #[serde(rename = "server")]
    ip_addr: IpAddr,
    port: u32,
    #[serde(rename = "type")]
    ptype: ProxyType,
}

#[derive(Debug, Serialize)]
struct ProxyGroups<'a> {
    name: &'a str,
    #[serde(rename = "type")]
    gtype: &'a str,
    proxies: Vec<&'a str>,
}

#[derive(Debug, Serialize)]
struct ProxyConfig<'a> {
    proxies: &'a Vec<Proxy>,
    #[serde(rename = "proxy-groups")]
    proxy_groups: [ProxyGroups<'a>; 1],
    rules: [&'a str; 1],
}

fn get_proxies(file_name: &Path) -> Vec<Proxy> {
    let mut proxies = HashSet::new();

    if file_name.is_dir() {
        return Vec::new();
    }

    println!("\nReading proxy info from: {}", file_name.display());

    let file = File::open(file_name).expect("file read error");
    let reader = BufReader::new(file);
    let mut line_cnt = 0;

    for line in reader.lines() {
        let line = line.unwrap();
        line_cnt = line_cnt + 1;

        if OPTIONS.verbose_level > 2 {
            println!("{}", line);
        }

        let parts: Vec<_> = line.split(':').collect();
        if parts.len() < 2 {
            println!("line error: {}", line);
            continue;
        }

        let ip_addr = parts[0];
        let port = u32::from_str(parts[1]).unwrap();
        let proxy_name = String::from_str(ip_addr).unwrap() + " - " + parts[1];
        let mut proxy_type = OPTIONS.ptype.clone();

        if parts.len() == 3 {
            proxy_type = ProxyType::from_str(parts[2]).unwrap();
        }

        proxies.insert(Proxy {
            name: proxy_name,
            ip_addr: match IpAddr::from_str(ip_addr) {
                Ok(addr) => addr,
                Err(..) => {
                    println!("ip address parse error at line: {}", line_cnt);
                    continue;
                }
            },
            port: port,
            ptype: proxy_type,
        });
    }

    Vec::from_iter(proxies)
}

fn write_to_file(file_path: &mut PathBuf, yaml_data: &ProxyConfig) {
    file_path.set_extension("yaml");

    println!("Write proxy info to: {}", file_path.display());

    match File::create(file_path) {
        Ok(file) => {
            if let Err(e) = serde_yml::to_writer(file, yaml_data) {
                println!("error occurs when write data to file: {}", e);
            }
        }

        Err(e) => {
            println!("error occurs when create file: {}", e);
        }
    }
}

fn generate_yaml(file_path: &PathBuf, proxies: &Vec<Proxy>) -> () {
    let mut proxy_names_in_group: Vec<&str> = Vec::with_capacity(proxies.len());

    // add proxy names to a list
    for proxy in proxies {
        proxy_names_in_group.push(&proxy.name);
    }

    let yaml_data = ProxyConfig {
        proxies: proxies,
        proxy_groups: [ProxyGroups {
            name: file_path.to_str().unwrap(),
            gtype: "select",
            proxies: proxy_names_in_group,
        }],
        rules: ["MATCH,DIRECT"],
    };

    write_to_file(&mut file_path.clone(), &yaml_data);
}

fn main() -> ExitCode {
    if 0 == OPTIONS.files_or_dirs.len() {
        return ExitCode::SUCCESS;
    }

    for fn_or_dirn in &OPTIONS.files_or_dirs {
        if fn_or_dirn.is_dir() {
            for entry in fs::read_dir(fn_or_dirn).unwrap() {
                let file_name = entry.unwrap().path();
                if let None = file_name.extension() {
                    generate_yaml(&file_name, &get_proxies(&file_name));
                }
            }
        } else {
            generate_yaml(fn_or_dirn, &get_proxies(fn_or_dirn));
        }
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {}
