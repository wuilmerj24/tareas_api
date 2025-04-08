#[macro_use] extern crate rocket;

use std::{collections::HashMap, sync::Arc};

use api::{AdduserResponse, Tareas, Usuarios};
use jsonwebtoken::{decode, DecodingKey, Validation};
use rocket::{fairing::{AdHoc, Fairing, Info, Kind}, futures::lock::Mutex, http::{Header, Status}, request::{self, FromRequest, Outcome}, routes, tokio::sync::broadcast::Sender, Request, Response};
use rocket_cors::{AllowedHeaders, AllowedMethods, AllowedOrigins, CorsOptions, Method};
use unreql::{cmd::connect::Options, r, Session};
use utils::Claims;
use utoipa::{openapi::security::{Http, HttpAuthScheme, HttpBuilder, SecurityScheme}, Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

mod utils;
mod api;

const API_URL: &str = "/v1/api";

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Tareas api",
        description = "API para gestión de tareas",
        contact(
            name = "Soporte Técnico",
            email = "wuilmermorgado24@gmail.com"
        ),
        license(
            name = "free",
            url = ""
        )
    ),
    servers(
        (url = "/v1/api/", description = "localhost"),
    ),
    paths(
        api::add_usuario,
        api::login,
        api::get_all_tareas,
        api::get_tarea_by_id,
        api::add_tarea,
        api::change_estado,
        api::delete_tarea_by_id,
        api::delete_tareas_by_id_user,
        api::connect_ws,
    ),
    components(
        schemas(
            Tareas,
            Usuarios,
            AdduserResponse
        )
    ), 
    modifiers(&SecurityAddon)
)]
struct ApiDoc;
// Añadir seguridad global
struct SecurityAddon;
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "jwt_token",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build()
                )
            );
        }
    }
}

pub struct SecurityHeaders;

#[rocket::async_trait]
impl Fairing for SecurityHeaders{
    fn info(&self) -> Info {
        Info {
            name: "Security Headers",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("X-Content-Type-Options", "nosniff"));
        response.set_header(Header::new("X-Frame-Options", "DENY"));
        response.set_header(Header::new("X-XSS-Protection", "1; mode=block"));
        response.set_header(Header::new("Content-Security-Policy", "default-src 'self'"));
        response.set_header(Header::new("Referrer-Policy", "no-referrer"));
        response.set_header(Header::new("Strict-Transport-Security", "max-age=31536000; includeSubDomains"));
        response.set_header(Header::new("Permissions-Policy", "geolocation=()"));
        response.set_header(Header::new("Cross-Origin-Embedder-Policy", "require-corp"));
    }

}

pub struct JwtGuard{
    pub id_usuario:String,   
}

#[rocket::async_trait]
impl <'r> FromRequest<'r> for JwtGuard {
    type Error = ();

    async fn from_request(request:&'r Request<'_>)->request::Outcome<Self,Self::Error>{
        let key = b"1234afeb";
        if let Some(aut_header) = request.headers().get_one("Authorization") {
            println!("✅ Header Authorization recibido: {}", aut_header);

            let token = aut_header.strip_prefix("Bearer ").unwrap_or("bearer no presente");
            println!("✅ Token extraído: {}", token);
            
            if !token.is_empty() {
                if let Ok(claims) = decode::<Claims>(token, &DecodingKey::from_secret(key), &Validation::default()) {
                    println!("✅ Claims válidos: {:?}", claims);
                    return Outcome::Success(JwtGuard { id_usuario: claims.claims.sub });
                } else {
                    println!("❌ Error al decodificar JWT");
                }
            } else {
                println!("❌ Token vacío");
            }
        } else {
            println!("❌ Header Authorization no encontrado");
        }


        Outcome::Error((Status::Unauthorized,()))
    }
    
}

pub type ClientesSockets = Arc<Mutex<HashMap<String,Sender<String>>>>;

#[rocket::main]
async fn main()->Result<(),rocket::Error> {    
    let conn:Session=r.connect(
        Options::new()
        .host("localhost")
        .port(28015)
        .db("tareas")
    )
    .await
    .expect("error conn rdb");


    let allowed_methods: AllowedMethods = vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]
        .into_iter()
        .map(|s| s.parse().unwrap())
        .collect();

    let allowed_headers = AllowedHeaders::some(&[
        "Authorization",
        "Content-Type",
        "Origin",
    ]);
    let allowed_origins:AllowedOrigins=AllowedOrigins::some_exact(&[
        "http://localhost",
        "http://localhost:8000",  // Si usas un puerto específico
        "http://127.0.0.1",      // Alternativa a localhost
        "http://127.0.0.1:8000",
    ]);
    let cors=CorsOptions {
        allowed_origins,
        allowed_methods,
        allowed_headers,
        allow_credentials: true,
        max_age: Some(600),
        ..Default::default()
    };

    rocket::build()
    .manage(conn.clone())
    .manage(Arc::new(Mutex::new(HashMap::new())) as ClientesSockets)
    .attach(cors.to_cors().unwrap())
    .attach(SecurityHeaders)
    .attach(AdHoc::on_liftoff("On Launch", |_|{
        Box::pin(async move{

        })
    }))
    .attach(AdHoc::on_shutdown("On Shutdown", |_| {
        Box::pin(async move {
            println!("🛑 Rocket se está apagando...");
        })
    }))
    .mount("/",SwaggerUi::new("/swagger-ui/<_..>").url("/api-docs/openapi.json", ApiDoc::openapi()),)
    .mount(format!("{}/usuarios",API_URL), routes![
        api::add_usuario,
        api::login,
    ])
    .mount(format!("{}/tareas",API_URL), routes![
        api::get_all_tareas,
        api::get_tarea_by_id,
        api::add_tarea,
        api::change_estado,
        api::delete_tarea_by_id,
        api::delete_tareas_by_id_user,
    ])
    .mount(format!("{}/ws",API_URL), routes![
        api::connect_ws,
    ])
    .launch()
    .await?;
    Ok(())
}