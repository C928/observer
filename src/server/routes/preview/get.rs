use crate::server::startup::PreviewHtmlContents;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use actix_web::{get, HttpResponse};

#[get("/preview")]
pub async fn preview(preview_html: Data<PreviewHtmlContents>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(preview_html.0.clone())
}
