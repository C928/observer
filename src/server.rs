use actix_files::Files;
use actix_web::{rt, App, HttpServer};
use actix_web_lab::web::redirect;
use anyhow::anyhow;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::net::TcpListener;
use std::sync::mpsc;

fn write_preview_html(
    server_ip: &str,
    port: u16,
    file_format: &str,
    interval: u16,
) -> anyhow::Result<()> {
    let contents = fs::read_to_string("preview_template.html")?
        .replacen('#', server_ip, 1)
        .replacen('#', &port.to_string(), 1)
        .replacen('#', file_format, 2)
        .replacen('#', &((interval as u64) * 1000 * 60).to_string(), 1);

    let mut fd = File::create("preview.html")?;
    write!(fd, "{}", contents)?;

    Ok(())
}

#[actix_web::main]
pub async fn start_server(
    server_ip: &String,
    host: &String,
    mut port: u16,
    file_format: &str,
    interval: u16,
    status_channels: Option<(&mpsc::Sender<String>, &crossbeam_channel::Receiver<()>)>,
) -> anyhow::Result<()> {
    let address = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&address)?;
    port = listener.local_addr()?.port();

    write_preview_html(server_ip, port, file_format, interval)?;
    let address_str = format!("Server address: {}:{}", server_ip, port);
    if let Some((tx, _)) = status_channels {
        if tx
            .send(format!(
                "Server started | {}\n{}",
                chrono::offset::Local::now().format("%H:%M:%S-%d/%m/%Y"),
                address_str
            ))
            .is_err()
        {
            return Err(anyhow!(
                "Error: Sending Server message through mpsc channel"
            ));
        }
    } else {
        println!("{address_str}");
    }

    let srv = HttpServer::new(|| {
        App::new()
            .service(redirect("/", "/preview/preview.html"))
            .service(Files::new("/preview", "."))
            .service(Files::new("/update", "."))
    })
    .listen(listener)?
    .run();

    let mut rt = None;
    if let Some((tx, rx)) = status_channels {
        let srv_handle = srv.handle();
        let tx = tx.clone();
        let rx = rx.clone();
        rt = Some(rt::spawn(async move {
            if rx.recv().is_err() {
                return Err(anyhow!(
                    "Error: Receiving shutdown message through crossbeam channel"
                ));
            }

            if tx
                .send(format!(
                    "Server stopped | {}",
                    chrono::offset::Local::now().format("%H:%M:%S-%d/%m/%Y")
                ))
                .is_err()
            {
                return Err(anyhow!(
                    "Error: Sending Server message through mpsc channel"
                ));
            }

            srv_handle.stop(false).await;
            Ok(())
        }));
    }

    srv.await?;
    if let Some(rt) = rt {
        rt.await??;
    }

    Ok(())
}
