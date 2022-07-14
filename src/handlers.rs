use crate::utils;

use super::AppError;

use log::{debug, info, warn};

use actix_web::{web, Responder};
use bollard::{
    container::{
        Config, CreateContainerOptions, KillContainerOptions, LogOutput, LogsOptions,
        StartContainerOptions, UpdateContainerOptions, UploadToContainerOptions,
    },
    Docker,
};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};
use tokio_stream::StreamExt;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunCodeRequest {
    pub base_64_tar_gz: String,
    pub max_time: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunCodeResponse {
    pub stdout: String,
    pub stderr: String,

}

#[actix_web::post("/run_code")]
pub async fn run_code(
    req: web::Json<RunCodeRequest>,
    docker: web::Data<Docker>,
) -> Result<impl Responder, AppError> {
    info!(target: "pythonbox::run_code", "recieved request");
    // convert base64 tar gz into bytes
    let content = base64::decode(req.base_64_tar_gz.as_str()).map_err(|_| {
        warn!(target: "pythonbox::run_code", "Invalid Base 64, refusing request");
        AppError::InvalidBase64
    })?;

    // create random string for container name
    let container_name = utils::random_string();

    // create docker container
    let container = docker
        .create_container(
            Some(CreateContainerOptions {
                name: container_name.as_str(),
            }),
            Config {
                image: Some("frolvlad/alpine-python3"),
                cmd: Some(vec!["/opt/run"]),
                network_disabled: Some(true),
                ..Default::default()
            },
        )
        .await
        .map_err(|_| {
            warn!(target:"pythonbox::docker", "couldn't create container!");
            AppError::InternalServerError
        })?;

    info!(target:"pythonbox::run_code", "created container {}", container_name.as_str());

    // set resource usage limits
    docker
        .update_container(
            container_name.as_str(),
            UpdateContainerOptions::<String> {
                ..Default::default()
            },
        )
        .await
        .map_err(|_| {
            warn!(target:"docker", "couldn't set resource usage limits!");
            AppError::InternalServerError
        })?;

    info!(target:"pythonbox::run_code", "limited container {}", container_name.as_str());

    // upload our tar.gz to the container
    docker
        .upload_to_container(
            container_name.as_str(),
            Some(UploadToContainerOptions {
                path: "/opt",
                no_overwrite_dir_non_dir: "false",
            }),
            content.into(),
        )
        .await
        .map_err(|_| {
            warn!(target:"docker", "couldn't upload data!");
            AppError::InternalServerError
        })?;

    info!(target:"pythonbox::run_code", "uploaded code to container {}", container_name.as_str());

    // start container
    docker
        .start_container(
            container_name.as_str(),
            None::<StartContainerOptions<String>>,
        )
        .await
        .map_err(|_| {
            warn!(target:"docker", "couldn't start container!");
            AppError::InternalServerError
        })?;

    info!(target:"pythonbox::run_code", "started container {}", container_name.as_str());

    // wait for the specified max_time
    sleep(Duration::from_secs_f32(req.max_time)).await;

    // kill container
    docker
        .kill_container(
            container_name.as_str(),
            None::<KillContainerOptions<String>>,
        )
        .await
        .map_err(|_| {
            warn!(target:"docker", "couldn't kill container!");
            AppError::InternalServerError
        })?;

    let mut logs = docker.logs(
        container_name.as_str(),
        Some(LogsOptions {
            stdout: true,
            stderr: true,
            tail: "all",
            ..Default::default()
        }),
    );

    // byte output
    let mut stdout = vec![];
    let mut stderr = vec![];

    while let Some(log_result) = logs.next().await {
        match log_result {
            Ok(LogOutput::StdErr { message }) => stderr.extend(message),
            Ok(LogOutput::StdOut { message }) => stdout.extend(message),
            _ => {}
        }
        info!(target:"pythonbox::run_code", "read log entry from {}", container_name.as_str());
    }


    Ok(web::Json("hi"))
}
