use rocket::serde::{Deserialize, Serialize};
use unreql::{cmd::options::InsertOptions, r, rjson, types::WriteStatus, Session};
use utoipa::ToSchema;
use validator::Validate;

use crate::utils::Utils;

#[derive(Debug,Clone,Deserialize,Serialize,ToSchema,Validate)]
#[serde(crate="rocket::serde")]
pub struct TareasHttp{
    pub nombre:String,
}

#[derive(Debug,Clone,Deserialize,Serialize,ToSchema,Validate)]
#[serde(crate="rocket::serde")]
pub struct Tareas{
    pub id:String,
    pub nombre:String,
    pub id_usuario:String,
    pub estado:bool,
    pub create_at:i64,
    pub update_at:i64,
}

impl Tareas {
    pub async fn get_all_tareas_by_usuario(id_usuario:&str,db:&Session)->Result<Vec<Tareas>,unreql::Error>{
        let query = r.db("tareas").table("tareas")
        .filter(rjson!({
            "id_usuario":id_usuario.to_string(),
        }))
        .exec_to_vec::<_,Tareas>(db);
        match query.await {
            Ok(res)=>{
                Ok(res)
            },
            Err(e)=>{
                println!("error get_all_tareas_by_usuario {}",e);
                Err(e)
            }
        }
    }
    
    pub async fn get_tarea_by_id(id:&str,db:&Session)->Result<Vec<Tareas>,unreql::Error>{
        let query = r.db("tareas").table("tareas")
        .get(id.to_string())
        .exec_to_vec::<_,Tareas>(db);
        match query.await {
            Ok(res)=>{
                Ok(res)
            },
            Err(e)=>{
                println!("error get_tarea_by_id {}",e);
                Err(e)
            }
        }
    }

    pub async fn change_estado(estado:&bool,id:&str,db:&Session)->Result<bool,unreql::Error>{
        let query = r.db("tareas").table("tareas")
        .get(id.to_string())
        .update(rjson!({
            "estado":estado.clone(),
            "update_at":Utils::current_timestamp(),
        }))
        .exec::<_,WriteStatus>(db);
        match query.await {
            Ok(res)=>{
                if res.replaced > 0 {
                    return Ok(true);
                }
                Ok(false)
            },
            Err(e)=>{
                println!("error change_estado {}",e);
                Err(e)
            }
        }
    }

    pub async fn add_tarea(tarea:&TareasHttp,id_usuario:&str,db:&Session)->Result<Vec<Tareas>,unreql::Error>{
        let query = r.db("tareas").table("tareas")
        .insert(
            r.with_opt(
                rjson!({
                    "nombre":tarea.nombre.clone(),
                    "id_usuario":id_usuario.to_string(),
                    "estado":false,
                    "create_at":Utils::current_timestamp(),
                    "update_at":Utils::current_timestamp(),
                }),
                InsertOptions {return_changes: Some(true.into()), ..Default::default() }
            )
        )
        .exec::<_,WriteStatus>(db);
        match query.await {
            Ok(res)=>{
                if res.inserted > 0 {
                    let changes = res.changes.ok_or("No hay cambios en la respuesta").unwrap();
                    let mut tareas: Vec<Tareas> = Vec::new();
                    for change in changes {
                        if let Some(new_val) = change.new_val {
                            let tarea: Tareas = Deserialize::deserialize(new_val)?;
                            tareas.push(tarea);
                        }
                    }
                    return Ok(tareas)
                }
                Ok(vec![])
            },
            Err(e)=>{
                println!("error add_tarea {}",e);
                Err(e)
            } 
        }
    }
    

    pub async fn delete_tarea_by_id(id:&str,db:&Session)->Result<bool,unreql::Error>{
        let query = r.db("tareas").table("tareas").get(id.to_string()).delete(()).exec::<_,WriteStatus>(db);
        match query.await {
            Ok(res)=>{
                if res.deleted > 0 {
                    return Ok(true);
                }
                Ok(false)  
            },
            Err(e)=>{
                println!("error delete_tarea_by_id {}",e);
                Err(e)
            }         
        }
    }

    pub async fn delete_all_tareas_by_usuario(id_usuario:&str,db:&Session)->Result<bool,unreql::Error>{
        let query = r.db("tareas").table("tareas")
        .filter(rjson!({
            "id_usuario":id_usuario.to_string()
        }))
        .delete(()).exec::<_,WriteStatus>(db);
        match query.await {
            Ok(res)=>{
                if res.deleted > 0 {
                    return Ok(true);
                }
                Ok(false)  
            },
            Err(e)=>{
                println!("error delete_tarea_by_id {}",e);
                Err(e)
            }         
        }
    }
}