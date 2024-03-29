use std::collections::HashMap;
use std::env;

use reqwest::{Client, header, StatusCode};
use sqlx::{Postgres, Row};
use tonic::{Request, Response, Status};
use tonic::metadata::MetadataMap;

use crate::config::locale;
use crate::proto::sms_service_server::SmsService;
use crate::proto::VerifyPhoneRequest;

rust_i18n::i18n!("locales");

pub struct SmsServiceImpl {
    db: sqlx::Pool<Postgres>,
}

impl SmsServiceImpl {
    pub fn new(db: sqlx::Pool<Postgres>) -> Self {
        Self {
            db,
        }
    }
}

#[tonic::async_trait]
impl SmsService for SmsServiceImpl {
    async fn send_phone_verification_code(&self, request: Request<String>) -> Result<Response<()>, Status> {
        let language_id = match _validate_language_id_from_request(request.metadata()) {
            Ok(language_id) => language_id,
            Err(e) => {
                return Err(e);
            }
        };

        // check if we are in dev mode
        // let is_dev = env::var("IS_DEV").expect("Error reading IS_DEV environment variable");
        // let is_dev = is_dev.parse::<bool>().unwrap_or(false);
        // if is_dev {
        //     log::info!("Skipping sms send in dev mode");
        //     return Ok(Response::new(()));
        // }

        // check if phone number already exists
        let phone_number = request.into_inner();
        log::info!("Sending sms to: {}", phone_number);

        let res = match sqlx::query("SELECT is_user_created_at_older_than_10_minutes($1)")
            .bind(&phone_number)
            .fetch_optional(&self.db)
            .await {
            Ok(res) => {
                match res {
                    Some(data) => {
                        match data.try_get(0) {
                            Ok(data) => data,
                            Err(e) => {
                                log::error!("Error getting data from database: {}", e);
                                false
                            }
                        }
                    }
                    None => false
                }
            }
            Err(e) => {
                log::error!("Error checking for existing phone number: {}", e);
                false
            }
        };

        if res {
            log::info!("Phone number already exists");
            return Err(Status::already_exists(t!("verification_already_exists", locale = &language_id)));
        }

        let res = match sqlx::query("SELECT insert_user($1)")
            .bind(&phone_number)
            .execute(&self.db)
            .await {
            Ok(data) => {
                data.rows_affected() == 1
            }
            Err(_) => false,
        };

        if !res {
            log::error!("Error inserting phone number");
            return Err(Status::internal(t!("sms_send_failed", locale = &language_id)));
        }

        let account_sid = env::var("TWILIO_ACCOUNT_SID").expect("Error reading Twilio Account SID");
        let auth_token = env::var("TWILIO_AUTH_TOKEN").expect("Error reading Twilio Auth Token");
        let service_id = env::var("TWILIO_SERVICES_ID").expect("Error reading Twilio Services ID");

        // create url
        let url = format!(
            "https://verify.twilio.com/v2/Services/{serv_id}/Verifications",
            serv_id = service_id
        );

        // create headers
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            "application/x-www-form-urlencoded".parse().unwrap(),
        );

        // create form body
        let mut form_body: HashMap<&str, String> = HashMap::new();
        form_body.insert("To", phone_number.to_owned());
        form_body.insert("Channel", "sms".to_string());

        let client = Client::new();
        let res = client
            .post(url)
            .basic_auth(account_sid, Some(auth_token))
            .headers(headers)
            .form(&form_body)
            .send()
            .await;

        match res {
            Ok(response) => {
                let created = response.status() == StatusCode::from_u16(201).unwrap();
                if created {
                    log::info!("{}", t!("sms_send_success", locale = &language_id));
                    Ok(Response::new(()))
                } else {
                    Err(Status::internal(t!("sms_send_failed", locale = &language_id)))
                }
            }
            Err(_) => Err(Status::internal(t!("sms_send_failed", locale = &language_id))),
        }
    }

    async fn verify_phone_verification_code(&self, request: Request<VerifyPhoneRequest>) -> Result<Response<()>, Status> {
        let language_id = match _validate_language_id_from_request(request.metadata()) {
            Ok(language_id) => language_id,
            Err(e) => {
                return Err(e);
            }
        };

        // check if we are in dev mode
        // let is_dev = env::var("IS_DEV").expect("Error reading IS_DEV environment variable");
        // let is_dev = is_dev.parse::<bool>().unwrap_or(false);
        // if is_dev {
        //     log::info!("Skipping sms verification in dev mode");
        //     return Ok(Response::new(()));
        // }

        let req = request.into_inner();
        let phone_number = req.phone_number;
        let code = req.verification_code;

        log::info!(
            "Verifying sms from phone number: {} -> {}",
            &phone_number,
            &code
        );
        let account_sid = env::var("TWILIO_ACCOUNT_SID").expect("Error reading Twilio Account SID");
        let auth_token = env::var("TWILIO_AUTH_TOKEN").expect("Error reading Twilio Auth Token");
        let service_id = env::var("TWILIO_SERVICES_ID").expect("Error reading Twilio Services ID");

        // create url
        let url = format!(
            "https://verify.twilio.com/v2/Services/{serv_id}/VerificationCheck",
            serv_id = service_id,
        );

        // create headers
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            "application/x-www-form-urlencoded".parse().unwrap(),
        );

        // create form body
        let mut form_body: HashMap<&str, &String> = HashMap::new();
        form_body.insert("To", &phone_number);
        form_body.insert("Code", &code);

        let client = Client::new();
        let res = client
            .post(url)
            .basic_auth(account_sid, Some(auth_token))
            .headers(headers)
            .form(&form_body)
            .send()
            .await;

        match res {
            Ok(response) => {
                let created = response.status() == StatusCode::from_u16(200).unwrap();
                if created {
                    log::info!("{}", t!("sms_verification_success", locale = &language_id));
                    let res = match sqlx::query("SELECT delete_user_by_phone_number($1)")
                        .bind(&phone_number)
                        .execute(&self.db)
                        .await {
                        Ok(data) => {
                            data.rows_affected() == 1
                        }
                        Err(_) => false,
                    };

                    if !res {
                        log::error!("Error inserting phone number");
                        return Err(Status::internal(t!("sms_send_failed", locale = &language_id)));
                    }
                    Ok(Response::new(()))
                } else {
                    Err(Status::internal(t!("sms_verification_failed", locale = &language_id)))
                }
            }
            Err(_) => {
                Err(Status::internal(t!("sms_verification_failed", locale = &language_id)))
            }
        }
    }
}

// validate language id
fn _validate_language_id_from_request(md: &MetadataMap) -> Result<String, Status> {
    log::info!("Validating language id from request => {:?}",md);
    let language_id = match md.get("x-language-id") {
        Some(result) => result.to_str().unwrap().to_string(),
        None => {
            return Err(Status::invalid_argument(t!("invalid_language_code")));
        }
    };
    // validate language id from request
    if let Ok(_) = locale::validate_language_id(&language_id) {
        rust_i18n::set_locale(&language_id);
        Ok(language_id)
    } else {
                return Err(Status::invalid_argument(t!("invalid_language_code")));
            }
}
