use std::net::Ipv4Addr;

use actix_web::http::StatusCode;
use actix_web::{error, App, HttpResponse, HttpServer};
use bollard::service::ProgressDetail;
use bollard::{image::CreateImageOptions, Docker};
use clap::Parser;
use derive_more::Display;
use log::info;
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

mod docker;
mod handlers;
mod utils;

#[derive(Clone, Debug, Serialize, Deserialize, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppError {
    DecodeError,
    InternalServerError,
    Unauthorized,
    BadRequest,
    NotFound,
    InvalidBase64,
    Unknown,
}

impl error::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self)
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            AppError::DecodeError => StatusCode::BAD_GATEWAY,
            AppError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::BadRequest => StatusCode::BAD_REQUEST,
            AppError::InvalidBase64 => StatusCode::BAD_REQUEST,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Parser, Debug, Clone)]
#[clap(about, version, author)]
struct Opts {
    #[clap(short, long)]
    port: u16,
    #[clap(short, long)]
    image: String,
}

#[derive(Debug, Clone)]
pub struct PythonboxData {
    pub docker: Docker,
    pub image: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    env_logger::init();

    let Opts { port, image } = Opts::parse();

    // create docker connection
    let docker = Docker::connect_with_local_defaults().unwrap();
    info!(target:"pythonbox::docker", "connected to docker!");

    // pull specified image
    info!(target:"pythonbox::docker", "pulling image: {}", &image);
    let mut pull_info_stream = docker.create_image(
        Some(CreateImageOptions {
            from_image: image.clone(),
            ..Default::default()
        }),
        None,
        None,
    );

    while let Some(mx) = pull_info_stream.next().await {
        let x = mx?;
        if let Some(ProgressDetail {
            current: Some(current),
            total: Some(total),
        }) = x.progress_detail
        {
            info!(target:"pythonbox:docker", "pull: {}/{}", current, total);
        }
        if let Some(status) = x.status {
            info!(target:"pythonbox:docker", "pull status: {}", status);
        }
    }

    /*
    let mut file = std::fs::File::open("testproj.tar.gz").unwrap();
    let mut contents = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut contents).unwrap();

    let x = docker::run_code(contents, 5.0, 100 * 0x100000, docker.clone())
        .await
        .unwrap();

    println!("stdout: {}", String::from_utf8_lossy(&x.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&x.stderr));
    println!("exit code: {:?}", x.exit_code);
    */

    // start server
    let data = PythonboxData { image, docker };
    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(data.clone()))
            .service(handlers::run_code)
    })
    .bind((Ipv4Addr::LOCALHOST, port))?
    .run()
    .await?;

    Ok(())
}
