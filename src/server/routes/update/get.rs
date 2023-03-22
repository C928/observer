use crate::server::startup::ObservedFileName;
use actix_files::NamedFile;
use actix_web::web::Data;
use actix_web::{get, Result};

#[get("/update")]
pub async fn update(observed_file: Data<ObservedFileName>) -> Result<NamedFile> {
    Ok(NamedFile::open_async(&observed_file.0).await?)
}
