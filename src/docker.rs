use bollard::{
    container::{
        Config, CreateContainerOptions, KillContainerOptions, LogOutput, LogsOptions,
        RemoveContainerOptions, StartContainerOptions, UpdateContainerOptions,
        UploadToContainerOptions,
    },
    models::ContainerState,
    Docker,
};
use log::{error, info};
use tokio::time::{sleep, Duration};
use tokio_stream::StreamExt;

use crate::{utils, AppError};

pub struct RunCodeResponse {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: Option<i64>,
}

pub async fn run_code(
    env_tar: Vec<u8>,
    max_time_s: f32,
    max_memory: i64,
    max_disk: i64,
    docker: Docker,
) -> Result<RunCodeResponse, AppError> {
    info!(target: "pythonbox::run_code", "recieved request");
    // create random string for container name
    let container_name = utils::random_string();

    // create docker container
    let _ = docker
        .create_container(
            Some(CreateContainerOptions {
                name: container_name.as_str(),
            }),
            Config {
                image: Some("frolvlad/alpine-python3"),
                cmd: Some(vec!["/opt/run"]),
                working_dir: Some("/opt"),
                network_disabled: Some(true),
                ..Default::default()
            },
        )
        .await
        .map_err(|_| {
            error!(target:"pythonbox::docker", "couldn't create container!");
            AppError::InternalServerError
        })?;

    info!(target:"pythonbox::run_code", "created container {}", container_name.as_str());

    // set resource usage limits
    docker
        .update_container(
            container_name.as_str(),
            UpdateContainerOptions::<String> {
                memory: Some(max_memory),
                // we don't want swapping! (this can severely degrade performance, better to just oom
                memory_swap: Some(max_memory),

                ..Default::default()
            },
        )
        .await
        .map_err(|_| {
            error!(target:"pythonbox::docker", "couldn't set resource usage limits!");
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
            env_tar.into(),
        )
        .await
        .map_err(|_| {
            error!(target:"pythonbox::docker", "couldn't upload data!");
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
        .map_err(|x| {
            dbg!(x);
            error!(target:"pythonbox::docker", "couldn't start container!");
            AppError::InternalServerError
        })?;

    info!(target:"pythonbox::run_code", "started container {}", container_name.as_str());

    // spawn an asynchronous tokio task to remove the docker container after a while.
    // In the meanwhile we'll be collecting log entries
    {
        let container_name = container_name.clone();
        let docker = docker.clone();
        tokio::task::spawn(async move {
            // wait for the specified max_time
            sleep(Duration::from_secs_f32(max_time_s)).await;

            // kill container
            let result = docker
                .kill_container(
                    container_name.as_str(),
                    None::<KillContainerOptions<String>>,
                )
                .await;
            info!(target:"pythonbox::run_code", "attempted to kill {}", container_name.as_str());
            if let Err(e) = result {
                info!(target:"pythonbox::run_code", "kill attempt failed, probably because already dead or removed {}", container_name.as_str());
            }
        });
    }

    let mut logs = docker.logs(
        container_name.as_str(),
        Some(LogsOptions {
            follow: true,
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

    // inspect container
    let inspect_result = docker
        .inspect_container(container_name.as_str(), None)
        .await
        .map_err(|_| {
            error!(target:"pythonbox::docker", "couldn't inspect container!");
            AppError::InternalServerError
        })?;

    let response = RunCodeResponse {
        stdout: stdout,
        stderr: stderr,
        exit_code: match inspect_result.state {
            Some(ContainerState { exit_code, .. }) => exit_code,
            None => None,
        },
    };

    // remove container
    docker
        .remove_container(
            container_name.as_str(),
            Some(RemoveContainerOptions {
                force: true,
                v: true,
                ..Default::default()
            }),
        )
        .await
        .map_err(|_| {
            error!(target:"pythonbox::docker", "couldn't remove container!");
            AppError::InternalServerError
        })?;

    return Ok(response);
}