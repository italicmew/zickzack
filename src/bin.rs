use actix_files::{Files, NamedFile};
use actix_web::{
    get,
    middleware::Logger,
    post,
    web::{self},
    App, Error, HttpResponse, HttpServer, Responder,
};
use errs::EditorError;
use path::generate_random_path;

use crate::db::{Data, Database};

mod db;
mod errs;
mod path;

#[get("/editor/{path:.*}")]
async fn editor(path: web::Path<String>) -> impl Responder {
    let connection_string = "files.db";
    match Database::new(connection_string) {
        Ok(db) => {
            db.create_db().map_err(|_| EditorError::ContentNotFound)?;

            match db.get_data(path.as_str().to_string()) {
                Ok(Some(data)) => Ok(HttpResponse::Ok().json(data)),
                Ok(None) => Ok(HttpResponse::NotFound().body("Data not found")),
                Err(_) => Err(EditorError::ContentNotFound),
            }
        }
        Err(_) => Err(EditorError::ContentNotFound),
    }
}

#[post("/editor/{path:.*}")]
async fn save_editor(data: web::Json<Data>) -> Result<HttpResponse, EditorError> {
    println!("{:?}", data);
    let connection_string = "files.db";
    match Database::new(connection_string) {
        Ok(db) => {
            db.create_db().map_err(|_| EditorError::ContentNotFound)?;
            db.save_data(&data.into_inner())
                .map_err(|_| EditorError::ContentNotFound)?;
            Ok(HttpResponse::Ok().body("Data saved successfully"))
        }
        Err(_) => Err(EditorError::ContentNotFound),
    }
}

#[get("/{tail:.*}")]
async fn index() -> Result<NamedFile, Error> {
    Ok(NamedFile::open("/bin/web/static/index.html")?)
}

async fn redirect_to_random_path() -> HttpResponse {
    let random_path = generate_random_path();
    HttpResponse::Found()
        .append_header(("Location", format!("/{}", random_path)))
        .finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    println!("Hello from server");
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .route("/", web::get().to(redirect_to_random_path))
            .service(editor)
            .service(save_editor)
            .service(Files::new("/static", "/bin/web/static").show_files_listing())
            .service(index)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
