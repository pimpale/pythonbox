use crate::{docker, PythonboxData};

use super::AppError;

use base64::engine;
use base64::Engine;
use log::error;

use actix_web::{web, Responder};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunCodeRequest {
    pub base_64_tar_gz: String,
    pub max_time_s: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunCodeResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i64>,
}

#[actix_web::post("/run_code")]
pub async fn run_code(
    req: web::Json<RunCodeRequest>,
    data: web::Data<PythonboxData>,
) -> Result<impl Responder, AppError> {
    // convert base64 tar gz into bytes
    let content = engine::general_purpose::STANDARD
        .decode(req.base_64_tar_gz.as_str())
        .map_err(|_| {
            error!(target: "pythonbox::run_code", "Invalid Base 64, refusing request");
            AppError::InvalidBase64
        })?;

    // max memory = 100MB
    let max_memory_usage = 100 * 0x100000;

    let resp = docker::run_code(
        content,
        req.max_time_s,
        max_memory_usage,
        data.image.clone(),
        data.docker.clone(),
    )
    .await?;

    let engine = engine::general_purpose::STANDARD;
    return Ok(web::Json(RunCodeResponse {
        stdout: engine.encode(resp.stdout),
        stderr: engine.encode(resp.stderr),
        exit_code: resp.exit_code,
    }));
}
