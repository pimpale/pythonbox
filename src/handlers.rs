use super::AppError;

use log::{info, warn};

use actix_web::{web, Responder};
use bollard::{container::UploadToContainerOptions, Docker};
use serde::{Deserialize, Serialize};






#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContactRequest {
    pub base64TarGz: String,
    pub max_time: f32,
}

#[actix_web::post("/run_code")]
pub async fn run_code(
    req: web::Json<ContactRequest>,
    docker: web::Data<Docker>,
) -> Result<impl Responder, AppError> {
    info!(target: "run_code", "recieved request");
    // convert base64 tar gz into bytes
    let content = base64::decode(req.base64TarGz.as_str()).map_err(|x| {
        warn!(target: "run_code", "Invalid Base 64, refusing request");
        AppError::InvalidBase64
    })?;

    // create random string
    //
    // ggj

    // create docker container
    docker.create_container(options, config);

    // upload our tar.gz to
    docker.upload_to_container(
        "my-container",
        Some(UploadToContainerOptions {
            path: "/opt",
            no_overwrite_dir_non_dir: "false",
        }),
        content.into(),
    ).map_err(
    );

    Ok(web::Json("hi"))
}
