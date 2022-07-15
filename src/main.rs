use std::net::Ipv4Addr;

use actix_web::http::StatusCode;
use actix_web::{error, App, HttpResponse, HttpServer};
use bollard::Docker;
use clap::Parser;
use derive_more::Display;
use log::info;
use serde::{Deserialize, Serialize};

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
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let Opts { port } = Opts::parse();

    // create docker connection
    let docker = Docker::connect_with_local_defaults().unwrap();
    info!(target:"pythonbox::docker", "connected to docker!");

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
    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(docker.clone()))
            .service(handlers::run_code)
    })
    .bind((Ipv4Addr::LOCALHOST, port))?
    .run()
    .await
}
