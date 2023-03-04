use crate::settings::{
    parse_file_format, parse_host, parse_server_ip, parse_target_browser, parse_url,
};
use clap::Parser;

#[derive(Parser)]
#[clap(name = "observer", author = "author: c928")]
pub struct Args {
    #[clap(
        short,
        long,
        takes_value = true,
        required = false,
        help = "URL of the website to observe\n(ex: https://www.gnu.org)",
        value_parser = parse_url,
    )]
    pub url: Option<String>,

    #[clap(
        short,
        long,
        takes_value = true,
        required = false,
        default_value = "5",
        help = "Interval between each browser capture\n"
    )]
    pub interval: u16,

    #[clap(
        short,
        long,
        takes_value = true,
        required = false,
        default_value = "jpeg",
        help = "File format of captures (jpeg/png)\n",
        value_parser = parse_file_format,
    )]
    pub file_format: String,

    #[clap(
        short,
        long,
        help_heading = "Flags",
        help = "If the quiet flag is enabled, observer\nwill not print logs to terminal in CLI\nmode"
    )]
    pub quiet: bool,

    #[clap(short, long, help_heading = "Flags", help = "Launch observer's GUI")]
    pub gui_mode: bool,

    #[clap(
        short = 'H',
        long,
        takes_value = true,
        required = false,
        default_value = "0.0.0.0",
        help = "IP address of the web server.\nHost can either be 127.0.0.1 to serve\nlocally on \
        the host computer or 0.0.0.0\nto serve on the computer network\n",
        value_parser = parse_host
    )]
    pub host: String,

    #[clap(
        short,
        long,
        takes_value = true,
        required = false,
        default_value = "0",
        help = "Port number used for the web server.\nIf not specified a random port will\n\
        be chosen by the OS and then printed\nto terminal (or GUI)\n"
    )]
    pub port: u16,

    #[clap(
        short,
        long,
        takes_value = true,
        required = false,
        help = "Private IP of the hosting server\n(ex: 192.168.13.37)",
        value_parser = parse_server_ip,
    )]
    pub server_ip: Option<String>,

    #[clap(
        short,
        long,
        help_heading = "Flags",
        help = "Use a configuration file (config.yaml)\nto read application settings\n"
    )]
    pub config_file: bool,

    #[clap(
        short = 'b',
        long,
        takes_value = true,
        required = false,
        default_value = "chrome",
        help = "Target browser used to capture the website.\nAvailable targets are: chromium, \
        chrome, edge\n",
        value_parser = parse_target_browser,
    )]
    pub target_browser: String,
}
