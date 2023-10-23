use std::{fs::{self, DirBuilder}, io, path::PathBuf};
use actix_files::{Files, NamedFile};
use actix_web::{HttpResponse, web::{self, Form}, get, post, HttpServer, App, Error, middleware::Logger};
use errs::EditorError;
use path::{generate_random_path, sanitize_path};
use serde_derive::Deserialize;

mod errs;
mod path;

#[get("/editor/{path:.*}")]
async fn editor(path: web::Path<String>) -> Result<HttpResponse, EditorError> {
    let content = load_content_from_path(&path.into_inner());
    Ok(HttpResponse::Ok().body(format!("{}", content)))
}

fn load_content_from_path(path: &str) -> String {
    let path = sanitize_path(path).unwrap(); //TODO: handle error
    fs::read_to_string(&path).unwrap_or_default()
}

fn create_parent_directory(path: &PathBuf) -> Result<(), EditorError> {
    if let Some(parent_dir) = std::path::Path::new(path).parent() {
        DirBuilder::new().recursive(true).create(parent_dir)
            .map_err(|_| EditorError::DirectoryCreationFailure)?;
    } else {
        return Err(EditorError::InvalidPath);
    }
    Ok(())
}

fn save_to_storage(storage_path: &PathBuf, content: &str) -> Result<(), EditorError> {
    fs::write(storage_path, content).map_err(|_| EditorError::SaveFailure)
}

#[post("/editor/{path:.*}")]
async fn save_editor(path: web::Path<String>, data: Form<ScratchContent>) -> Result<HttpResponse, EditorError> {
    let storage_path = sanitize_path(path.as_str()).unwrap();

    match fs::write(&storage_path, &data.content) {
        Ok(_) => Ok(HttpResponse::Ok().body("Saved successfully!")),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            create_parent_directory(&storage_path)?;
            match save_to_storage(&storage_path, &data.content) {
                Ok(_) => Ok(HttpResponse::Ok().body("Saved successfully!")),
                Err(_) => Err(EditorError::SaveFailure),
            }
        },
        Err(_) => Err(EditorError::SaveFailure),
    }
}

#[derive(Deserialize)]
struct ScratchContent {
    content: String,
}

#[get("/{tail:.*}")]
async fn index() -> Result<NamedFile, Error> {
    Ok(NamedFile::open("/bin/web/static/index.html")?)
}

async fn redirect_to_random_path() -> HttpResponse {
    let random_path = generate_random_path();
    HttpResponse::Found().append_header(("Location", format!("/{}", random_path))).finish()
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