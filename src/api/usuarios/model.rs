use rocket::serde::{Deserialize, Serialize};
use unreql::{r, rjson, types::WriteStatus, DateTime, Session};
use utoipa::ToSchema;
use validator::Validate;

use crate::utils::Utils;

#[derive(Debug,Clone,Deserialize,Serialize,ToSchema,Validate)]
#[serde(crate="rocket::serde")]
pub struct UsuariosHttp{
    pub username:String,
    pub password:String,
}

#[derive(Debug,Clone,Deserialize,Serialize,ToSchema,Validate)]
#[serde(crate="rocket::serde")]
pub struct Usuarios{
    pub id:String,
    pub username:String,
    pub password:String,
    pub create_at:i64,
}

#[derive(Debug,Clone,Deserialize,Serialize,ToSchema,Validate)]
#[serde(crate="rocket::serde")]
pub struct AdduserResponse{
    pub error:i8,
}

impl Usuarios {
    pub async fn filter_by_username(username:&str,db:&Session)->Result<Vec<Usuarios>,unreql::Error>{
        let query = r.db("tareas").table("usuarios").filter(rjson!({
            "username":username.to_string(),
        }))
        .exec_to_vec::<_,Usuarios>(db);
        match query.await {
            Ok(res)=>{
                Ok(res)
            },
            Err(e)=>{
                println!("error filter_by_email {}",e);
                Err(e)
            }
        }
    }

    pub async fn add(user:&UsuariosHttp,db:&Session)->Result<bool,unreql::Error>{
        let password_hash=match Utils::hash_password_argon2d(&user.password){
            Ok(res)=>res,
            Err(e)=>{
                println!("error hash password {}",e);
                e.to_string()
            }
        };
        let query = r.db("tareas").table("usuarios").insert(rjson!({
            "username":user.username.clone(),
            "password":password_hash.clone(),
            "create_at":Utils::current_timestamp()
        }))
        .exec::<_,WriteStatus>(db);
        match query.await {
            Ok(res)=>{
                if res.inserted >  0 {
                    return Ok(true)
                }
                Ok(false)
            },
            Err(e)=>{
                println!("error  add user {}",e);
                Err(e)
            }
            
        }
    }
}