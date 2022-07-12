use super::AppError;

use actix_web::{web, Responder};
use mail_service_api::client::MailService;
use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContactRequest {
    pub name: String,
    pub email: String,
    pub school: String,
    pub position: String,
    pub message: String,
}

#[actix_web::post("/public/send_contact_email")]
pub async fn send_contact_email(
    req: web::Json<ContactRequest>,
    mail_service_url: web::Data<String>,
) -> Result<impl Responder, AppError> {
    let mail_service = MailService::new(&mail_service_url).await;

    let _ = mail_service
        .mail_new(mail_service_api::request::MailNewProps {
            request_id: 0,
            destination: "innexgo@gmail.com".to_owned(),
            topic: "Innexgo Sales: New Contact".to_owned(),
            title: "New contact from form:<br/>".to_owned(),
            content: format!(
                "name: <code>{}</code><br/>
                email: <code>{}</code><br/>
                school: <code>{}</code><br/>
                position: <code>{}</code><br/> +
                message: <code>{}</code><br/>",
                req.name, req.email, req.school, req.position, req.message
            ),
        })
        .await;

    Ok(web::Json("hi"))
}
