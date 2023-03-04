use gtk::glib::{timeout_add_seconds_local, Char, OptionArg, OptionFlags};
use gtk::prelude::*;
use gtk::{glib, Align, Application, ApplicationWindow, Orientation};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;

use crate::observer::start_observing;
use crate::server::start_server;
use crate::settings::{
    load_config_gui_mode, parse_config_gui_mode, parse_file_format, parse_host, parse_server_ip,
    parse_target_browser, parse_url, Settings,
};

pub fn start_gui() {
    let app = Application::builder()
        .application_id("com.github.c928.observer")
        .build();

    app.add_main_option(
        "gui-mode",
        Char::from(b'g'),
        OptionFlags::NONE,
        OptionArg::None,
        "",
        None,
    );

    app.connect_activate(move |app| {
        build_ui(app);
    });

    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Observer")
        .build();
    window.set_default_width(542);

    let main_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .spacing(8)
        .build();

    let left_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .margin_start(16)
        .margin_end(16)
        .build();

    let observer_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .margin_bottom(8)
        .spacing(8)
        .build();

    let server_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .margin_bottom(8)
        .spacing(8)
        .build();

    let right_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .margin_start(16)
        .margin_end(16)
        .build();

    let config_box = gtk::Box::builder()
        .valign(Align::Center)
        .halign(Align::Center)
        .build();

    let shared_input_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .margin_top(8)
        .margin_bottom(16)
        .spacing(8)
        .build();

    let message_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .margin_top(8)
        .spacing(8)
        .build();

    let quiet_box = gtk::Box::builder().margin_top(8).margin_start(24).build();

    let url = gtk::Entry::builder()
        .input_purpose(gtk::InputPurpose::Url)
        .placeholder_text("https://gnu.org")
        .build();
    observer_box.append(&gtk::Label::new(Some("URL")));
    observer_box.append(&url);

    let target_browser = gtk::Entry::builder().placeholder_text("chromium").build();
    observer_box.append(&gtk::Label::new(Some("Target Browser")));
    observer_box.append(&target_browser);

    let server_ip = gtk::Entry::builder()
        .placeholder_text("192.168.13.37")
        .build();
    server_box.append(&gtk::Label::new(Some("Server IP")));
    server_box.append(&server_ip);

    let host = gtk::Entry::builder().placeholder_text("0.0.0.0").build();
    server_box.append(&gtk::Label::new(Some("Host Address")));
    server_box.append(&host);

    let port_number = gtk::SpinButton::with_range(0.0, 49151.0, 1.0);
    port_number.set_value(0.0);
    server_box.append(&gtk::Label::new(Some("Port Number")));
    server_box.append(&port_number);

    left_box.append(&observer_box);
    let observer_button = gtk::Button::builder()
        .label("Start Observer")
        .margin_top(4)
        .margin_bottom(8)
        .build();
    left_box.append(&observer_button);

    left_box.append(
        &gtk::Separator::builder()
            .orientation(Orientation::Horizontal)
            .margin_top(4)
            .margin_bottom(8)
            .build(),
    );

    left_box.append(&server_box);
    let server_button = gtk::Button::builder()
        .label("Start Server")
        .margin_top(4)
        .margin_bottom(4)
        .build();
    left_box.append(&server_button);

    let load_config_ch_btn = gtk::CheckButton::builder()
        .halign(Align::Center)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(12)
        .margin_end(16)
        .build();
    config_box.append(&load_config_ch_btn);

    let load_config_btn = gtk::Button::builder()
        .label("Load Config")
        .margin_top(8)
        .margin_bottom(8)
        .build();
    config_box.append(&load_config_btn);

    let scrolled_win_status_msg = gtk::ScrolledWindow::new();
    scrolled_win_status_msg.set_size_request(256, 128);

    shared_input_box.append(
        &gtk::Separator::builder()
            .orientation(Orientation::Horizontal)
            .margin_bottom(4)
            .build(),
    );

    let file_format = gtk::Entry::builder()
        .max_length(4)
        .placeholder_text("JPEG")
        .build();
    shared_input_box.append(&gtk::Label::new(Some("File Format (JPEG/PNG)")));
    shared_input_box.append(&file_format);

    let interval = gtk::SpinButton::with_range(0.0, u16::MAX as f64, 1.0);
    interval.set_value(5.0);
    shared_input_box.append(&gtk::Label::new(Some("Saving Interval (minute)")));
    shared_input_box.append(&interval);

    let status_msg = Rc::new(RefCell::new(
        gtk::TextView::builder()
            .cursor_visible(false)
            .editable(false)
            .build(),
    ));
    scrolled_win_status_msg.set_child(Some(&*status_msg.borrow()));
    message_box.append(&gtk::Label::new(Some("Status / Error messages")));
    message_box.append(&scrolled_win_status_msg);

    let quiet_flag_ch_btn = gtk::CheckButton::builder()
        .halign(Align::Center)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(12)
        .margin_end(16)
        .active(true)
        .build();
    quiet_box.append(&quiet_flag_ch_btn);
    quiet_box.append(&gtk::Label::new(Some("Print Observer captures")));

    right_box.append(&gtk::Label::new(Some("Load Config File (config.yaml)")));
    // for the fun of it :)
    let boxes = [&config_box, &shared_input_box, &message_box, &quiet_box];
    for i in 0..5 {
        if i != 2 {
            right_box.append(if i < 2 { boxes[i] } else { boxes[i - 1] });
        } else {
            right_box.append(&gtk::Separator::new(Orientation::Horizontal));
        }
    }

    main_box.append(&left_box);
    main_box.append(&gtk::Separator::new(Orientation::Horizontal));
    main_box.append(&right_box);

    let settings = Rc::new(RefCell::new(Settings::default()));
    load_config_btn.connect_clicked(glib::clone!(@weak settings, @weak status_msg => move |_| {
        *settings.borrow_mut() = match load_config_gui_mode() {
            Ok(s) => {
                append_status_msg(
                    &status_msg,
                    "Configuration successfully loaded from config.yaml".to_owned()
                );
                s
            }
            Err(e) => {
                append_status_msg(&status_msg, e);
                return;
            }
        };
    }));

    let (status_msg_tx, status_msg_rx) = mpsc::channel();
    let (observer_stop_tx, observer_stop_rx) = crossbeam_channel::bounded(1);
    observer_button.connect_clicked(glib::clone!(
        @weak load_config_ch_btn,
        @weak settings,
        @weak status_msg,
        @strong status_msg_tx,
        @weak file_format,
        @weak interval => move |observer_button| {
        match observer_button.label().expect("Reading observer_button label").as_str() {
            "Start Observer" => {
                let inter;
                let quiet_flag = !quiet_flag_ch_btn.is_active();
                let load_config_is_active = load_config_ch_btn.is_active();
                let mut config = match load_config_is_active {
                    true => {
                        inter = settings.borrow().interval;
                        [
                            settings.borrow().target_browser.clone(),
                            settings.borrow().url.clone(),
                            settings.borrow().file_format.clone(),
                        ]
                    },
                    false => {
                        inter = interval.value_as_int() as u16;
                        [
                            target_browser.text().to_string(),
                            url.text().to_string(),
                            file_format.text().to_string(),
                        ]
                    }
                };

                if let Err(e) = parse_config_gui_mode(
                    &mut config,
                    [parse_target_browser, parse_url, parse_file_format],
                    true,
                    load_config_is_active
                ) {
                    append_status_msg(&status_msg, e);
                    return;
                }

                let status_msg_tx_c = status_msg_tx.clone();
                let observer_stop_rx_c = observer_stop_rx.clone();
                std::thread::spawn(move || {
                    if let Err(e) = start_observing(
                        &config[0],
                        &config[1],
                        &config[2],
                        inter,
                        quiet_flag,
                        Some((&status_msg_tx_c, &observer_stop_rx_c)),
                    ) {
                        status_msg_tx_c
                            .send(format!("[err-obs]{}", e))
                            .expect("Sending Observer error through mpsc channel");
                    }
                });
                observer_button.set_label("Stop Observer")
            }
            "Stop Observer" => {
                observer_button.set_label("Start Observer");
                observer_stop_tx.send(()).expect("Stopping Observer with crossbeam channel");
            }
            _ => (),
        };
    }));

    let (server_stop_tx, server_stop_rx) = crossbeam_channel::bounded(1);
    server_button.connect_clicked(glib::clone!(
        @weak load_config_ch_btn,
        @weak status_msg => move |server_button| {
        match server_button.label().expect("Reading server_button label").as_str() {
            "Start Server" => {
                let mut nums = (0, 5);
                let load_config_is_active = load_config_ch_btn.is_active();
                let mut config = match load_config_is_active {
                    true => {
                        nums.0 = settings.borrow().port;
                        nums.1 = settings.borrow().interval;
                        [
                            settings.borrow().server_ip.clone(),
                            settings.borrow().host.clone(),
                            settings.borrow().file_format.clone(),
                        ]
                    },
                    false => {
                        nums.0 = port_number.value_as_int() as u16;
                        nums.1 = interval.value_as_int() as u16;
                        [
                            server_ip.text().to_string(),
                            host.text().to_string(),
                            file_format.text().to_string(),
                        ]
                    },
                };

                if let Err(e) = parse_config_gui_mode(
                    &mut config,
                    [parse_server_ip, parse_host, parse_file_format],
                    false,
                    load_config_is_active
                ) {
                    append_status_msg(&status_msg, e);
                    return;
                }

                let status_msg_tx_c = status_msg_tx.clone();
                let server_stop_rx_c = server_stop_rx.clone();
                std::thread::spawn(move || {
                    if let Err(e) = start_server(
                        &config[0],
                        &config[1],
                        nums.0,
                        &config[2],
                        nums.1,
                        Some((&status_msg_tx_c, &server_stop_rx_c)),
                    ) {
                        status_msg_tx_c
                            .send(format!("[err-serv]{}", e))
                            .expect("Sending Server error through mpsc channel");
                    }
                });
                server_button.set_label("Stop Server");
            }
            "Stop Server" => {
                server_button.set_label("Start Server");
                server_stop_tx.send(()).expect("Stopping Server with crossbeam channel");
            }
            _ => (),
        }
    }));

    timeout_add_seconds_local(2, move || {
        if let Ok(mut msg) = status_msg_rx.try_recv() {
            if msg.starts_with("[err-obs]") {
                msg = msg[9..].to_owned();
                observer_button.set_label("Start Observer");
            } else if msg.starts_with("[err-serv]") {
                msg = msg[10..].to_owned();
                server_button.set_label("Start Server");
            }
            append_status_msg(&status_msg, msg);
        }

        Continue(true)
    });

    load_config_ch_btn.connect_active_notify(move |load_config_ch_btn| {
        let mut visibility = (true, true, true);
        if load_config_ch_btn.is_active() {
            if observer_box.is_visible() {
                visibility.0 = false;
            }
            if server_box.is_visible() {
                visibility.1 = false;
            }
            if shared_input_box.is_visible() {
                visibility.2 = false;
            }
        }

        observer_box.set_visible(visibility.0);
        server_box.set_visible(visibility.1);
        shared_input_box.set_visible(visibility.2);
    });

    window.set_child(Some(&main_box));
    window.show();
}

fn append_status_msg(status_msg: &Rc<RefCell<gtk::TextView>>, msg: String) {
    let status_msg = status_msg.borrow().buffer();
    if status_msg
        .text(&status_msg.start_iter(), &status_msg.end_iter(), true)
        .as_str()
        .len()
        + msg.len()
        > 65536
    {
        status_msg.set_text(&msg);
    } else {
        status_msg.set_text(&format!(
            "{}{}\n---\n",
            status_msg
                .text(&status_msg.start_iter(), &status_msg.end_iter(), true)
                .as_str(),
            msg
        ));
    }
}
