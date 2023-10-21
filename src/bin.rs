use std::{fs::{self, DirBuilder}, io, path::{Path, PathBuf}};
use actix_files::{Files, NamedFile};
use actix_web::{HttpResponse, web::{self, Form}, get, post, HttpServer, App, Error, HttpRequest};
use errs::EditorError;
use serde_derive::Deserialize;

mod errs;

fn sanitize_path(input: &str) -> Result<PathBuf, EditorError> {
    let base  = Path::new("./data");
    // Step 1: Remove problematic sequences
    let sanitized_input = input
        .replace("..", "")
        .replace("//", "/");

    // Step 2: Construct a path from the sanitized input
    let path = base.join(sanitized_input);

    // Step 3: Check if the path is still within the base directory
    if !path.starts_with(base) {
        return Err(EditorError::InvalidPath);
    }

    Ok(path)
}

#[get("/editor/{path:.*}")]
async fn editor(path: web::Path<String>) -> Result<HttpResponse, EditorError> {
    let content = load_content_from_path(&path.into_inner());
    Ok(HttpResponse::Ok().body(format!("<textarea>{}</textarea>", content)))
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

#[get("/")]
async fn index(req: HttpRequest) -> Result<NamedFile, Error> {
    Ok(NamedFile::open("/bin/web/static/index.html")?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Hello from server");
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(editor)
            .service(save_editor)
            .service(Files::new("/static", "/bin/web/pkg").show_files_listing())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}