use crate::server::startup::ObservedFileName;
use actix_web::web::Data;
use actix_web::{get, HttpRequest, HttpResponse};

#[get("/update")]
pub async fn update(req: HttpRequest, observed_file: Data<ObservedFileName>) -> HttpResponse {
    match actix_files::NamedFile::open_async(&observed_file.0).await {
        Ok(file) => file.into_response(&req),
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}
