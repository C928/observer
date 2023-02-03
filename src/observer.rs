use anyhow::anyhow;
use chrono;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::protocol::cdp::Target::CreateTarget;
use headless_chrome::Browser;
use std::process::Command;
use std::sync::mpsc;
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs};

pub fn start_observing(
    target_browser: &str,
    url: &String,
    file_format: &String,
    interval: u16,
    quiet_flag: bool,
    status_channels: Option<(&mpsc::Sender<String>, &crossbeam_channel::Receiver<()>)>,
) -> anyhow::Result<()> {
    let starting_timestamp = chrono::offset::Local::now().format("%H:%M:%S-%d/%m/%Y");
    if let Some((tx, _)) = status_channels {
        if tx
            .send(format!("Observer started | {}", starting_timestamp))
            .is_err()
        {
            return Err(anyhow!(
                "Error: Sending Observer message through mpsc channel"
            ));
        }
    } else {
        let s = if interval == 1 { "" } else { "s" };
        println!(
            "{} | Observer started\nURL: {}\nFile Format: {}\nBrowser Capture Interval: {} minute{}",
            starting_timestamp,
            url,
            file_format,
            interval,
            s,
        );
    }

    let interval = interval as u64 * 60;
    let status_msg_tx = status_channels.map(|(tx, _)| tx);
    loop {
        browser_capture(target_browser, url, file_format, quiet_flag, status_msg_tx)?;
        if let Some((tx, rx)) = status_channels {
            if rx.recv_timeout(Duration::from_secs(interval)).is_ok() {
                let stopping_timestamp = chrono::offset::Local::now().format("%H:%M:%S-%d/%m/%Y");
                if tx
                    .send(format!("Observer stopped | {}", stopping_timestamp))
                    .is_err()
                {
                    return Err(anyhow!(
                        "Error: Sending Observer message through mpsc channel"
                    ));
                }

                return Ok(());
            }
        } else {
            sleep(Duration::from_secs(interval));
        }
    }
}

fn browser_capture(
    target_browser: &str,
    url: &String,
    file_format: &str,
    quiet_flag: bool,
    status_msg_tx: Option<&mpsc::Sender<String>>,
) -> anyhow::Result<()> {
    let filename = format!("./observed.{}", file_format);
    match &*target_browser.to_lowercase() {
        "chromium" | "chrome" => {
            // catch potential panic from headless_chrome crate as it does not return any error
            match std::panic::catch_unwind(|| {
                std::panic::set_hook(Box::new(|_| {
                    // hook potential panic to avoid printing panic message to terminal
                }));

                let browser = Browser::default()?;
                let url_str = url.to_owned();
                let tab = browser.new_tab_with_options(CreateTarget {
                    url: url_str,
                    width: Some(1920),
                    height: Some(1080),
                    browser_context_id: None,
                    enable_begin_frame_control: None,
                    new_window: None,
                    background: None,
                })?;

                if let Err(e) = tab.navigate_to(url) {
                    return Err(anyhow!("{}\nCapture of {} failed", e, url));
                }
                tab.wait_until_navigated()?;

                let fmt = match &*file_format.to_lowercase() {
                    "png" => CaptureScreenshotFormatOption::Png,
                    "jpeg" => CaptureScreenshotFormatOption::Jpeg,
                    _ => return Err(anyhow!("Error: file format must either be JPEG or PNG")),
                };

                let image_data = tab.capture_screenshot(fmt, Some(100), None, false)?;
                fs::write(&filename, image_data)?;

                // unregister panic hook
                let _ = std::panic::take_hook();
                Ok(())
            }) {
                Ok(ret) => ret?,
                Err(_) => {
                    return Err(anyhow!(
                        "Error: Capture of {} failed.\nMake sure chrome or chromium is installed",
                        url
                    ))
                }
            }
        }
        "edge" => {
            if env::consts::OS != "windows" {
                return Err(anyhow!(
                    "Error: Edge target only supported on Windows.\nOS detected: {}",
                    env::consts::OS
                ));
            }

            let screenshot_arg = format!(
                "--screenshot={}{}",
                env::current_dir()?
                    .to_str()
                    .ok_or_else(|| anyhow!("Error: Reading current directory"))?,
                filename
            );

            if !Command::new("C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge")
                .args([
                    "--headless",
                    "--disable-gpu",
                    "--window-size=1920,1080",
                    &screenshot_arg,
                    url,
                ])
                .status()?
                .success()
            {
                return Err(anyhow!("Error: Failed to capture {}", url));
            }
        }
        browser => {
            return Err(anyhow!("Error: Unsupported target browser '{}'", browser));
        }
    }

    if !quiet_flag {
        let saving_timestamp = chrono::offset::Local::now().format("%H:%M:%S-%d/%m/%Y");
        if let Some(tx) = status_msg_tx {
            if tx
                .send(format!("{} | {}", filename, saving_timestamp))
                .is_err()
            {
                return Err(anyhow!(
                    "Error: Sending Observer message through mpsc channel"
                ));
            }
        } else {
            println!("{} | {} saved", saving_timestamp, filename);
        }
    }

    Ok(())
}
