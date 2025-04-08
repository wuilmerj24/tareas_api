use rocket::{futures::future::ok, http::Status, post, response::status::{self, Custom}, serde::json::Json, State};
use unreql::Session;
use validator::Validate;

use crate::utils::Utils;

use super::{AdduserResponse, Usuarios, UsuariosHttp};

#[utoipa::path(
    post,
    path = "/usuarios/",
    responses(
        (status = 201, description = "Agregado correctamente", body = AdduserResponse),
        (status = 409, description = "El username ya está registrado",body = AdduserResponse),
        (status = 400, description = "Datos requeridos faltantes o mal formateadas",body = AdduserResponse),
        (status = 500, description = "Error en la query",body = AdduserResponse),
    ),
    request_body=UsuariosHttp,
    tag="usuarios"
)]
#[post("/",data="<user>")]
pub async fn add_usuario(db:&State<Session>,user:Json<UsuariosHttp>)->Result<Json<AdduserResponse>,status::Custom<Json<AdduserResponse>>>{
    if let Err(errors) = user.clone().validate(){
        return Err(
            status::Custom(
                Status::BadRequest,
                Json(
                    AdduserResponse { error: 4 }
                )
            )
        );
    }

    match Usuarios::filter_by_username(&user.username,&db).await {
        Ok(res)=>{
            if res.len() <=0{
                println!("usuarios {}",res.len());
                match Usuarios::add(&user.clone(),&db).await {
                    Ok(res_add)=>{
                        println!("rees_add {}",res_add);
                        return Ok(
                            Json(
                                AdduserResponse { 
                                    error:0
                                }
                            )
                        )
                    },
                    Err(e_add)=>{
                        println!("e_add {}",e_add);
                        return Err(Custom(
                            Status::InternalServerError,
                            Json(
                                AdduserResponse{
                                    error:1
                                }
                            )
                        ))
                    }
                }
            }else{
                return Err(Custom(
                    Status::Conflict,
                    Json(
                        AdduserResponse{
                            error:1
                        }
                    )
                ))
            }            
        },
        Err(e)=>{
            Err(Custom(
                Status::InternalServerError,
                Json(
                    AdduserResponse{
                        error:2
                    }
                )
            ))
        }
    }
}


#[utoipa::path(
    post,
    path = "/usuarios/login",
    responses(
        (status = 200, description = "Ok Login", body = String),
        (status = 401, description = "email o pass incorrecto",body = AdduserResponse),
        (status = 500, description = "Error en la query",body = AdduserResponse)
    ),
    request_body= UsuariosHttp,
    tag="usuarios"
)]
#[post("/login",data="<user>")]
pub async fn login(user:Json<UsuariosHttp>,db:&State<Session>)->Result<String,status::Custom<Json<AdduserResponse>>>{
    if let Err(errors) = user.clone().validate(){
        return Err(
            status::Custom(
                Status::BadRequest,
                Json(
                    AdduserResponse { error: 1 }
                )
            )
        );
    }

    match Usuarios::filter_by_username(&user.username,&db).await{
        Ok(usuario)=>{
            if usuario.len() > 0{
                println!("usuario.len() {} ",usuario.clone().get(0).unwrap().clone().id);
                let pass_ok=match Utils::verify_password_argon2d(&user.password,&usuario.clone().get(0).unwrap().password.clone()) {
                    Ok(res)=>res,
                    Err(e)=>false
                };
                println!("pass_ok {}",pass_ok);
                if pass_ok {
                    match Utils::make_jwt(&usuario.clone().get(0).unwrap().clone().username,&usuario.clone().get(0).unwrap().clone().id).await {
                        Ok(token)=>{
                            return Ok(token);
                        },
                        Err(e)=>{
                            return Err(
                                status::Custom(
                                    Status::InternalServerError,
                                    Json(
                                        AdduserResponse { error: 2 }
                                    )
                                )
                            );
                        }
                        
                    };
                }else{
                    return Err(
                        status::Custom(
                            Status::InternalServerError,
                            Json(
                                AdduserResponse { error: 2 }
                            )
                        )
                    );
                }
            }else{
                return Err(
                    status::Custom(
                        Status::BadRequest,
                        Json(
                            AdduserResponse { error: 3 }
                        )
                    )
                );
            }
        },
        Err(e)=>{
            Err(Custom(
                Status::InternalServerError,
                Json(
                    AdduserResponse{
                        error:4
                    }
                )
            ))
        }
    }
}
