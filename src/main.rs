use std::net::Ipv4Addr;

use log::{info};
use actix_web::http::StatusCode;
use actix_web::{error, App, HttpResponse, HttpServer};
use bollard::Docker;
use clap::Parser;
use serde::{Deserialize, Serialize};

mod handlers;
use derive_more::Display;
use handlers::run_code;

#[derive(Clone, Debug, Serialize, Deserialize, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppError {
    DecodeError,
    InternalServerError,
    Unauthorized,
    BadRequest,
    NotFound,
    InvalidBase64,
    InvalidTarGz,
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
            AppError::InvalidTarGz => StatusCode::BAD_REQUEST,
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
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let Opts { port } = Opts::parse();

    let docker = Docker::connect_with_local_defaults().unwrap();
    info!(target:"docker", "connected to docker!");
    docker.ping().unwrap();
    info!(target:"docker", "able to ping docker!");
    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(docker.clone()))
            .service(run_code)
    })
    .bind((Ipv4Addr::LOCALHOST, port))?
    .run()
    .await
}