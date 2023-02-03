use observer::gui;
use observer::observer::start_observing;
use observer::server::start_server;
use observer::utils::Settings;

fn main() -> anyhow::Result<()> {
    let s = Settings::parse_settings()?;
    if s.gui_mode {
        gui::start_gui();
    } else {
        let ff = s.file_format.clone();
        std::thread::spawn(move || {
            if let Err(e) = start_observing(
                &s.target_browser,
                &s.url,
                &ff,
                s.interval,
                s.quiet_flag,
                None,
            ) {
                eprintln!("{e}");
                std::process::exit(1);
            }
        });

        start_server(
            &s.server_ip,
            &s.host,
            s.port,
            &s.file_format,
            s.interval,
            None,
        )?;
    }

    Ok(())
}

//todo: reduce size of shared_input_box widgets (gui.rs)
//todo: reduce size of window when load_config is active (gui.rs)

//todo: optional: ===================================
//todo: ui files with build script
//todo: tls
//todo: add /favicon.ico
//todo: diagrams
