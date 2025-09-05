use actix_web::{App, HttpRequest, HttpServer, web};

mod config;

async fn index(req: HttpRequest) -> &'static str {
    println!("Received request: {}", req.path());
    "Hello, World!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = config::load_config();
    println!("Port: {}", config.port);

    HttpServer::new(|| App::new().service(web::resource("/").to(index)))
        .bind(format!("127.0.0.1:{}", config.port))?
        .run()
        .await
}

#[cfg(test)]
mod tests {
    use actix_web::{Error, body::to_bytes, dev::Service, http, test};

    use super::*;

    #[actix_web::test]
    async fn test_index() -> Result<(), Error> {
        let app = App::new().route("/", web::get().to(index));
        let app = test::init_service(app).await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = app.call(req).await?;

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = resp.into_body();
        assert_eq!(to_bytes(response_body).await?, r##"Hello, World!"##);

        Ok(())
    }
}
