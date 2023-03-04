use crate::cli::Args;
use anyhow::anyhow;
use clap::Parser;
use config::{Config, ConfigError, File, FileFormat};
use std::io::{Error, ErrorKind};

#[derive(Default, serde::Deserialize)]
pub struct Settings {
    pub server_ip: String,
    pub host: String,
    pub port: u16,
    pub target_browser: String,
    pub url: String,
    pub file_format: String,
    pub interval: u16,
    pub quiet_flag: bool,
    pub gui_mode: bool,
}

impl Settings {
    pub fn parse_settings() -> anyhow::Result<Self> {
        let args = Args::parse();
        if args.config_file {
            return match read_config_file() {
                Ok(mut s) => {
                    if args.gui_mode {
                        s.gui_mode = true;
                    }

                    parse_config_file([
                        &mut s.server_ip,
                        &mut s.host,
                        &mut s.target_browser,
                        &mut s.file_format,
                    ])?;

                    Ok(s)
                }
                Err(e) => Err(e.into()),
            };
        }

        if !args.gui_mode {
            if args.server_ip.is_none() {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "--server-ip must be specified\nFor more information try --help",
                )
                .into());
            } else if args.url.is_none() {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "--url must be specified\nFor more information try --help",
                )
                .into());
            }
        } else {
            return Ok(Self {
                gui_mode: true,
                ..Default::default()
            });
        }

        Ok(Self {
            host: args.host,
            server_ip: args.server_ip.unwrap(),
            port: args.port,
            target_browser: args.target_browser,
            url: args.url.unwrap(),
            file_format: args.file_format,
            interval: args.interval,
            quiet_flag: args.quiet,
            gui_mode: args.gui_mode,
        })
    }
}

pub fn load_config_gui_mode() -> Result<Settings, String> {
    let mut s = match read_config_file() {
        Ok(s) => s,
        Err(e) => return Err(e.to_string()),
    };

    s.gui_mode = true;
    s.file_format = parse_file_format(&s.file_format)?;
    s.target_browser = parse_target_browser(&s.target_browser)?;
    s.url = parse_url(&s.url)?;
    s.host = parse_host(&s.host)?;

    Ok(s)
}

pub fn parse_file_format(f: &str) -> Result<String, String> {
    let format = f.trim().to_lowercase();
    if !["png", "jpeg", "jpg"].contains(&&*format) {
        return Err("file format must either be JPEG or PNG".to_owned());
    }

    Ok(format)
}

pub fn parse_target_browser(t: &str) -> Result<String, String> {
    let target = t.trim().to_lowercase();
    if !["chromium", "chrome", "edge"].contains(&&*target) {
        return Err("capture targets available are chromium, chrome and edge".to_owned());
    }

    Ok(target)
}

pub fn parse_url(u: &str) -> Result<String, String> {
    let url = u.trim().to_lowercase();
    if !url.starts_with("https://") && !url.starts_with("http://") || !validator::validate_url(&url)
    {
        return Err("url must follow this format: '[https,http]://some_domain.com'".to_owned());
    }

    Ok(url)
}

pub fn parse_host(h: &str) -> Result<String, String> {
    let host = h.trim().to_lowercase();
    if !["127.0.0.1", "0.0.0.0"].contains(&&*host) {
        return Err(
            "host can either be 127.0.0.1 to serve locally on the host computer\n\
        or 0.0.0.0 to serve on the computer network"
                .to_owned(),
        );
    }

    Ok(host)
}

pub fn parse_server_ip(i: &str) -> Result<String, String> {
    let server_ip = i.trim().to_lowercase();
    if !validator::validate_ip_v4(&server_ip) {
        return Err("server IP must be an IPv4 address".to_owned());
    }

    Ok(server_ip)
}

fn parse_config_file(str_config: [&mut String; 4]) -> anyhow::Result<()> {
    let parsing_func = [
        parse_server_ip,
        parse_host,
        parse_target_browser,
        parse_file_format,
    ];
    for i in 0..4 {
        match parsing_func[i](str_config[i]) {
            Ok(ret) => {
                str_config[i].clear();
                str_config[i].push_str(&ret);
            }
            Err(e) => {
                return Err(anyhow!("{e}"));
            }
        }
    }

    Ok(())
}

type ParsingFunc = fn(&str) -> Result<String, String>;
pub fn parse_config_gui_mode(
    str_config: &mut [String; 3],
    parsing_func: [ParsingFunc; 3],
    observer_button: bool,
    config_file_mode: bool,
) -> Result<(), String> {
    for c in &mut *str_config {
        if c.is_empty() {
            let mut err_str = "Error: these fields must be filled before\nstarting the ".to_owned();
            if observer_button {
                err_str += "observer\n[URL, Target Browser, File Format, Saving Interval]";
            } else {
                err_str +=
                    "server\n[File Format, Saving Interval, Server IP, Host Address, Port Number]";
            }

            if config_file_mode {
                err_str += "\nMake sure to load configuration file (Load Config button)";
            }

            return Err(err_str);
        }
    }

    for i in 0..3 {
        match parsing_func[i](&str_config[i]) {
            Ok(ret) => {
                str_config[i].clear();
                str_config[i].push_str(&ret);
            }
            Err(e) => {
                return Err(format!("Error: {}", e));
            }
        }
    }

    Ok(())
}

fn read_config_file() -> Result<Settings, ConfigError> {
    Config::builder()
        .add_source(File::new("config", FileFormat::Yaml))
        .build()?
        .try_deserialize()
}
