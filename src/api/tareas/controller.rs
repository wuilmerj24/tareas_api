use std::{collections::HashMap, sync::Arc};

use rocket::{futures::lock::Mutex, http::Status, response::status::{self, Custom}, serde::json::Json, tokio::sync::broadcast::Sender, State};
use unreql::Session;

use crate::{api::{AdduserResponse, Tareas}, utils::Utils, ClientesSockets, JwtGuard};

use super::TareasHttp;

#[utoipa::path(
    get,
    path = "/tareas/",
    responses(
        (status = 200, description = "Existen tareas", body = Vec<Tareas>),
        (status = 404, description = "No tiene tareas", body =AdduserResponse),
        (status = 500, description = "Error interno del servidor", body = AdduserResponse),
        (status = 401, description = "No autorizado")
    ),
    security(
        ("jwt_token" = [])
    ),
    tag="tareas"
)]
#[get("/")]
pub async fn get_all_tareas(userClaims:JwtGuard,db:&State<Session>)->Result<Custom<Json<Vec<Tareas>>>,status::Custom<Json<AdduserResponse>>>{
    match Tareas::get_all_tareas_by_usuario(&userClaims.id_usuario,&db).await {
        Ok(res)=>{
            if res.len() > 0 {
                return Ok(
                    Custom(
                        Status::Ok, 
                        Json(res)
                    )
                );
            }

            Err(
                Custom(
                    Status::NotFound,
                    Json(
                        AdduserResponse{
                            error:1
                        }
                    )
                )
            )
        },
        Err(e)=>{
            Err(
                Custom(
                    Status::InternalServerError,
                    Json(
                        AdduserResponse{
                            error:2
                        }
                    )
                )
            )
        }
    }
}


#[utoipa::path(
    get,
    path = "/tareas/{id}",
    responses(
        (status = 200, description = "Tarea encontrada", body = Vec<Tareas>),
        (status = 404, description = "Tarea no encontrada", body =AdduserResponse),
        (status = 500, description = "Error interno del servidor", body = AdduserResponse),
        (status = 401, description = "No autorizado")
    ),
    params(
        ("id" =String, Path, description = "id de la tarea  a obtener")
    ),
    security(
        ("jwt_token" = [])
    ),
    tag="tareas"
)]
#[get("/<id>")]
pub async fn get_tarea_by_id(userClaims:JwtGuard,db:&State<Session>,id:&str)->Result<Custom<Json<Vec<Tareas>>>,status::Custom<Json<AdduserResponse>>>{
    match Tareas::get_tarea_by_id(&id,&db).await {
        Ok(res)=>{
            if res.len() > 0 {
                return Ok(
                    Custom(
                        Status::Ok, 
                        Json(res)
                    )
                );
            }

            Err(
                Custom(
                    Status::NotFound,
                    Json(
                        AdduserResponse{
                            error:1
                        }
                    )
                )
            )
        },
        Err(e)=>{
            Err(
                Custom(
                    Status::InternalServerError,
                    Json(
                        AdduserResponse{
                            error:2
                        }
                    )
                )
            )
        }
    }
}

#[utoipa::path(
    post,
    path = "/tareas/",
    responses(
        (status = 201, description = "Tarea creada", body = String),
        (status = 400, description = "Datos inválidos", body = AdduserResponse),
        (status = 500, description = "Error interno del servidor", body = AdduserResponse),
        (status = 401, description = "No autorizado")
    ),
    request_body=Tareas,
    security(
        ("jwt_token" = [])
    ),
    tag="tareas"
)]
#[post("/",data="<tarea>")]
pub async fn add_tarea(db:&State<Session>,userClaims:JwtGuard,tarea:Json<TareasHttp>,clients:&State<ClientesSockets>)->Result<Custom<Json<Vec<Tareas>>>,status::Custom<Json<AdduserResponse>>>{
    match Tareas::add_tarea(&tarea, &userClaims.id_usuario, &db).await {
        Ok(res)=>{
            if res.len() > 0 {
                let clients:Arc<Mutex<HashMap<String,Sender<String>>>>=clients.inner().clone();
                Utils::send_ws(&clients,"new").await;
                return Ok(
                    Custom(
                        Status::Ok, 
                        Json(res)
                    )
                );
            }else{
                Err(
                    Custom(
                        Status::BadRequest,
                        Json(
                            AdduserResponse{
                                error:1
                            }
                        )
                    )
                )  
            }
        },
        Err(e)=>{
            Err(
                Custom(
                    Status::InternalServerError,
                    Json(
                        AdduserResponse{
                            error:2
                        }
                    )
                )
            )
        }
    }
}

#[utoipa::path(
    put,
    path = "/tareas/{id}",
    responses(
        (status = 201, description = "estado tarea cambiado", body = String),
        (status = 400, description = "Id tarea no encontado", body = AdduserResponse),
        (status = 500, description = "Error interno del servidor", body = AdduserResponse),
        (status = 401, description = "No autorizado")
    ),
    params(
        ("id" =String, Path, description = "id de la tarea  a obtener")
    ),
    security(
        ("jwt_token" = [])
    ),
    tag="tareas"
)]
#[put("/<id>")]
pub async fn change_estado(db:&State<Session>,id:&str,userClaims:JwtGuard,clients:&State<ClientesSockets>)->Result<Custom<String>,status::Custom<Json<AdduserResponse>>>{
    match Tareas::change_estado(&true,id, &db).await {
        Ok(res)=>{
            if res{
                let clients:Arc<Mutex<HashMap<String,Sender<String>>>>=clients.inner().clone();
                Utils::send_ws(&clients,"update").await;
                return Ok(
                    Custom(
                        Status::Ok, 
                        "Ok ".to_string()
                    )
                );
            }
            Err(
                Custom(
                    Status::InternalServerError,
                    Json(
                        AdduserResponse{
                            error:2
                        }
                    )
                )
            )
        },
        Err(e)=>{
            Err(
                Custom(
                    Status::BadRequest,
                    Json(
                        AdduserResponse{
                            error:1
                        }
                    )
                )
            )
        }     
    }
}

#[utoipa::path(
    delete,
    path = "/tareas/{id}",
    responses(
        (status = 201, description = "tarea eliminada", body = String),
        (status = 400, description = "Id tarea no encontado", body = AdduserResponse),
        (status = 500, description = "Error interno del servidor", body = AdduserResponse),
        (status = 401, description = "No autorizado")
    ),
    params(
        ("id" =String, Path, description = "id de la tarea  a obtener")
    ),
    security(
        ("jwt_token" = [])
    ),
    tag="tareas"
)]
#[delete("/<id>")]
pub async fn delete_tarea_by_id(db:&State<Session>,id:&str,userClaims:JwtGuard,clients:&State<ClientesSockets>)->Result<Custom<String>,status::Custom<Json<AdduserResponse>>> {
    match Tareas::delete_tarea_by_id(&id, &db).await {
        Ok(res)=>{
            if res{

                let clients:Arc<Mutex<HashMap<String,Sender<String>>>>=clients.inner().clone();
                Utils::send_ws(&clients,"delete_one").await;
                return Ok(
                    Custom(
                        Status::Ok, 
                        "Ok ".to_string()
                    )
                );
            }
            Err(
                Custom(
                    Status::InternalServerError,
                    Json(
                        AdduserResponse{
                            error:2
                        }
                    )
                )
            )
        },
        Err(e)=>{
            Err(
                Custom(
                    Status::BadRequest,
                    Json(
                        AdduserResponse{
                            error:1
                        }
                    )
                )
            )
        }     
    }
    
}

#[utoipa::path(
    delete,
    path = "/tareas/",
    responses(
        (status = 201, description = "tarea eliminada", body = String),
        (status = 400, description = "Id tarea no encontado", body = AdduserResponse),
        (status = 500, description = "Error interno del servidor", body = AdduserResponse),
        (status = 401, description = "No autorizado")
    ),
    security(
        ("jwt_token" = [])
    ),
    tag="tareas"
)]
#[delete("/")]
pub async fn delete_tareas_by_id_user(db:&State<Session>,userClaims:JwtGuard,clients:&State<ClientesSockets>)->Result<Custom<String>,status::Custom<Json<AdduserResponse>>> {
    match Tareas::delete_all_tareas_by_usuario(&userClaims.id_usuario,&db).await {
        Ok(res)=>{
            if res{
                let clients:Arc<Mutex<HashMap<String,Sender<String>>>>=clients.inner().clone();
                Utils::send_ws(&clients,"delete_all").await;
                return Ok(
                    Custom(
                        Status::Ok, 
                        "Ok ".to_string()
                    )
                );
            }
            Err(
                Custom(
                    Status::InternalServerError,
                    Json(
                        AdduserResponse{
                            error:2
                        }
                    )
                )
            )
        },
        Err(e)=>{
            Err(
                Custom(
                    Status::BadRequest,
                    Json(
                        AdduserResponse{
                            error:1
                        }
                    )
                )
            )
        }     
    }
    
}