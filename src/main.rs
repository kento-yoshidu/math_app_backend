use actix_web::{App, HttpResponse, HttpServer, get};

#[get("/api/units")]
async fn list_units() -> HttpResponse {
    HttpResponse::Ok().json(Vec::<()>::new())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(list_units))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
