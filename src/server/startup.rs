use crate::server::routes::{preview, update};
use actix_web::web::Data;
use actix_web::{rt, App, HttpServer};
use actix_web_lab::web::redirect;
use anyhow::anyhow;
use std::net::TcpListener;
use std::sync::mpsc;

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

    let preview_html = Data::new(PreviewHtmlContents::new(
        server_ip,
        port,
        file_format,
        interval,
    ));
    let observed_file_format = Data::new(ObservedFileName(format!("observed.{}", file_format)));
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

    let srv = HttpServer::new(move || {
        App::new()
            .service(redirect("/", "/preview"))
            .service(preview)
            .service(update)
            .app_data(preview_html.clone())
            .app_data(observed_file_format.clone())
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

pub struct ObservedFileName(pub String);

pub struct PreviewHtmlContents(pub String);
impl PreviewHtmlContents {
    fn new(server_ip: &str, port: u16, file_format: &str, interval: u16) -> Self {
        Self(
            include_str!("preview_template.html")
                .replacen('#', server_ip, 1)
                .replacen('#', &port.to_string(), 1)
                .replacen('#', file_format, 1)
                .replacen('#', &((interval as u64) * 1000 * 60).to_string(), 1),
        )
    }
}
