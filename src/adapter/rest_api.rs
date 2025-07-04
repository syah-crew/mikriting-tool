use actix_files as fs;
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use actix_web::{
    cookie::Key, 
    web, 
    App, 
    Error, 
    HttpRequest, 
    HttpResponse, 
    HttpServer, 
    Responder,
    Result,
    middleware::Logger,
};
use actix_web_actors::ws;
use actix::{Actor, Addr};
use clap::Parser;
use log::{info, debug, error};
use serde::Deserialize;
use std::net::IpAddr;
use std::sync::Arc;

use crate::domain::traits::ConfigService;
use crate::usecase::{VpnUserUseCase, AuthUseCase};
use crate::adapter::websocket::{WebSocketActor, WebSocketManager};

#[derive(Debug, Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Listen address
    #[arg(short, long, default_value = "127.0.0.1")]
    address: IpAddr,
    /// Listen port
    #[arg(short, long, default_value_t = 3217)]
    port: u16,
}

struct AppState {
    vpn_user_use_case: Arc<VpnUserUseCase>,
    auth_use_case: Arc<AuthUseCase>,
    websocket_manager: Addr<WebSocketManager>,
}

// Route handlers
async fn index(session: Session, _data: web::Data<AppState>) -> impl Responder {
    match session.get::<String>("username") {
        Ok(Some(_)) => {
            // Serve the index.html file
            match std::fs::read_to_string("./asset/index.html") {
                Ok(content) => HttpResponse::Ok()
                    .content_type("text/html; charset=utf-8")
                    .body(content),
                Err(_) => HttpResponse::InternalServerError()
                    .body("Failed to load index.html"),
            }
        }
        _ => HttpResponse::SeeOther()
            .append_header(("Location", "/login"))
            .finish(),
    }
}

async fn login_page() -> impl Responder {
    match std::fs::read_to_string("./asset/login.html") {
        Ok(content) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(content),
        Err(_) => HttpResponse::InternalServerError()
            .body("Failed to load login.html"),
    }
}

async fn login(
    form: web::Form<LoginForm>,
    session: Session,
    data: web::Data<AppState>,
) -> impl Responder {
    let LoginForm { username, password } = form.into_inner();
    
    match data.auth_use_case.authenticate(&username, &password).await {
        Ok(auth_user) if auth_user.is_authenticated => {
            if let Err(e) = session.insert("username", &username) {
                error!("Failed to create session: {}", e);
                return HttpResponse::InternalServerError().body("Session error");
            }
            
            debug!("User {} logged in successfully", username);
            HttpResponse::SeeOther()
                .append_header(("Location", "/"))
                .finish()
        }
        Ok(_) => {
            debug!("Authentication failed for user: {}", username);
            HttpResponse::SeeOther()
                .append_header(("Location", "/login?error=1"))
                .finish()
        }
        Err(e) => {
            error!("Login error: {}", e);
            HttpResponse::InternalServerError().body("Authentication error")
        }
    }
}

async fn logout(session: Session) -> impl Responder {
    if let Ok(Some(username)) = session.get::<String>("username") {
        debug!("User {} logged out", username);
    }
    
    session.purge();
    HttpResponse::SeeOther()
        .append_header(("Location", "/login"))
        .finish()
}

async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    session: Session,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    match session.get::<String>("username")? {
        Some(_) => {
            let websocket_actor = WebSocketActor::new(data.websocket_manager.clone());
            ws::start(websocket_actor, &req, stream)
        }
        None => Ok(HttpResponse::Unauthorized().body("Unauthorized")),
    }
}

async fn trigger_update(data: web::Data<AppState>) -> impl Responder {
    match data.vpn_user_use_case.fetch_and_update_users().await {
        Ok(users) => {
            info!("Manual update triggered, fetched {} users", users.len());
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": format!("Updated {} users", users.len())
            }))
        }
        Err(e) => {
            error!("Manual update failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("Update failed: {}", e)
            }))
        }
    }
}

async fn get_users(data: web::Data<AppState>) -> impl Responder {
    match data.vpn_user_use_case.get_all_users().await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => {
            error!("Failed to get users: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch users"
            }))
        }
    }
}

async fn disconnect_user(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let username = path.into_inner();
    
    match data.vpn_user_use_case.disconnect_user(&username).await {
        Ok(()) => {
            info!("User {} disconnected", username);
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": format!("User {} disconnected", username)
            }))
        }
        Err(e) => {
            error!("Failed to disconnect user {}: {}", username, e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("Failed to disconnect user: {}", e)
            }))
        }
    }
}

fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/", web::get().to(index))
        .route("/login", web::get().to(login_page))
        .route("/login", web::post().to(login))
        .route("/logout", web::get().to(logout))
        .route("/ws", web::get().to(websocket_handler))
        .route("/api/trigger-update", web::post().to(trigger_update))
        .route("/api/users", web::get().to(get_users))
        .route("/api/users/{username}/disconnect", web::post().to(disconnect_user))
        .service(fs::Files::new("/static", "./asset").show_files_listing());
}

pub async fn start_server(
    vpn_user_use_case: Arc<VpnUserUseCase>,
    auth_use_case: Arc<AuthUseCase>,
    config_service: Arc<dyn ConfigService + Send + Sync>,
) -> std::io::Result<()> {
    let args = Args::parse();
    
    let app_config = config_service.get_app_config()
        .expect("Failed to get app config");
    
    info!("Starting mikriting-tool server on http://{}:{}", args.address, args.port);
    info!("Static files served from: {}", app_config.static_files_path);
    
    let websocket_manager = WebSocketManager::new().start();
    
    let app_state = web::Data::new(AppState {
        vpn_user_use_case,
        auth_use_case,
        websocket_manager,
    });
    
    let secret_key = Key::generate();
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .configure(configure_routes)
    })
    .bind((args.address, args.port))?
    .run()
    .await
}
