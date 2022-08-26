use std::convert::Infallible;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::server::conn::AddrIncoming;
use hyper::service::{make_service_fn, service_fn};
use serenity::client::Context;
use serenity::model::gateway::Ready;
use tracing::{/*error, */info};

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

pub(crate) async fn backend_http_handler(req: Request<Body>, cloned_ctx: Context) -> Result<Response<Body>, hyper::Error> {
    info!("method: {} - {}", req.method(), req.uri().path());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/debug") => {
            let channel = cloned_ctx.http.clone().get_channel(1012167414571618348).await.unwrap();

            channel.id().send_message(&cloned_ctx.http, |m| {
                m.content(format!("method: {} path: {}\nmeow", req.method(), req.uri().path()))
            }).await.expect("Failed to send message");

            Ok(Response::new(Body::from(channel.to_string())))
        }

        (&Method::POST, "/echo") => Ok(Response::new(req.into_body())),

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