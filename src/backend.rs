use std::convert::Infallible;
use hyper::{Body, HeaderMap, Method, Request, Response, Server, StatusCode};
use hyper::server::conn::AddrIncoming;
use hyper::service::{make_service_fn, service_fn};
use serenity::client::Context;
use serenity::model::gateway::Ready;
use tracing::{error, info};

pub(crate) async fn spawn_backend(context: Context, _ready: Ready) -> Result<(), hyper::Error> {
    let http_addr = ([0, 0, 0, 0], 7273).into();

    let cloned_ctx = context.clone();

    let http_service = make_service_fn(move |_| {
        let cloned_ctx = cloned_ctx.clone();
        async move {
            let meow = service_fn(move |req| backend_http_handler(req, cloned_ctx.clone()));
            Ok::<_, Infallible>(meow)
        }
    });

    let incoming = AddrIncoming::bind(&http_addr).unwrap_or_else(|e| {
        panic!("error binding to {}: {}", &http_addr, e);
    });

    // server with uppercase headers
    let http_server = Server::builder(incoming)
        .http1_preserve_header_case(true)
        .http1_title_case_headers(true)
        .serve(http_service);

    info!("backend | Listening on {}://{}", "http", http_addr);

    http_server.await?;

    Ok::<(), hyper::Error>(())
}

/*
fn get_access_token() -> String {
    let access_token = json::parse(std::fs::read_to_string(".twitch_access_token").unwrap().as_str()).unwrap();

    println!("{}", access_token["access_token"].as_str().unwrap());

    access_token["access_token"].as_str().unwrap().to_string()
}
*/

fn get_hmac_message(headers: &HeaderMap, body: String) -> String {
    let message_id = headers.get("Twitch-Eventsub-Message-Id").unwrap().to_str().unwrap().to_string();
    let message_timestamp = headers.get("Twitch-Eventsub-Message-Timestamp").unwrap().to_str().unwrap().to_string();

    let mut hmac_message = String::new();
    hmac_message.push_str(&message_id);
    hmac_message.push_str(&message_timestamp);
    hmac_message.push_str(&body);

    hmac_message
}

fn get_hmac(message: String) -> [u8; 32] {
    let secret = std::fs::read_to_string(".meow_secret").unwrap().as_str().to_string();

    let mut hmac = hmac_sha256::HMAC::new(secret);
    hmac.update(message.as_bytes());
    hmac.finalize()
}

fn verify_message(headers: &HeaderMap, body: String) -> bool {
    let message = get_hmac_message(headers, body);

    let mut hmac = String::new();
    hmac.push_str("sha256=");
    for &byte in get_hmac(message).as_ref() {
        hmac.push_str(&format!("{:02x}", byte));
    }

    let twitch_hmac = headers.get("Twitch-Eventsub-Message-Signature").unwrap().to_str().unwrap().to_string();
    //println!("a {}", hmac);
    //println!("b {}", twitch_hmac);

    // return
    hmac == twitch_hmac
}

pub(crate) async fn backend_http_handler(req: Request<Body>, cloned_ctx: Context) -> Result<Response<Body>, hyper::Error> {
    info!("method: {} - {}", req.method(), req.uri().path());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/debug") => {
            /*let channel = cloned_ctx.http.clone().get_channel(1012187714734530591).await.unwrap();

            channel.id().send_message(&cloned_ctx.http, |m| {
                m.content("meow!")
            }).await.expect("Failed to send message");*/

            Ok(Response::new(Body::from("{}")))
        }

        (&Method::POST, "/twitch/sub/callback") => {
            let headers = req.headers().clone();

            let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
            let body = String::from_utf8(body_bytes.to_vec()).unwrap();

            let mut response;

            info!("{}", body);

            if verify_message(&headers, body.clone()) {
                let json_data = json::parse(body.as_str()).unwrap();

                if headers.get("Twitch-Eventsub-Message-Type").is_some() &&
                    headers.get("Twitch-Eventsub-Message-Type").unwrap().to_str().unwrap() == "webhook_callback_verification" {

                    let challenge = json_data["challenge"].as_str().unwrap().to_string();

                    response = Response::new(Body::from(challenge));
                } else {


                    let event_type = json_data["subscription"]["type"].as_str().expect("No event type");
                    let broadcaster_user_id = json_data["event"]["broadcaster_user_id"].as_str().expect("No broadcaster id");
                    let broadcaster_user_login = json_data["event"]["broadcaster_user_login"].as_str().expect("No broadcaster user login");
                    let broadcaster_user_name = json_data["event"]["broadcaster_user_name"].as_str().expect("No broadcaster user name");

                    let channel = cloned_ctx.http.clone().get_channel(1012187714734530591).await.unwrap();

                    match event_type {
                        "stream.online" => {
                            if broadcaster_user_id == "576291377" {
                                channel.id().send_message(&cloned_ctx.http, |m| {
                                    m.content(&format!("<@&{}>\nI went live! come watch me at https://twitch.tv/{}", "1012179040372789258", broadcaster_user_login))
                                }).await.expect("Failed to send message");
                            } else {
                                error!("{} - someone that does not actually belong here started streaming!!! (wtf how) (real) (cops called)", broadcaster_user_name);
                                /*
                                channel.id().send_message(&cloned_ctx.http, |m| {
                                    m.content(&format!("))
                                }).await.expect("Failed to send message");
                                */
                            }
                        }
                        _ => {
                            error!("UNSUPPORTED EVENT - {}", event_type);
                        }
                    }

                    response = Response::new(Body::from("{\"status\": \"ok\"}"));
                }
            } else {
                response = Response::new(Body::from("{\"status\": \"error\"}"));
                *response.status_mut() = StatusCode::BAD_REQUEST;
            }

            Ok(response)
        },

        // return 404 for all other requests
        _ => {
            let mut not_found = Response::new(Body::from(
                "Not Found",
            ));
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}