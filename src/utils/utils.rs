use std::{collections::HashMap, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use argon2::{
    password_hash::{self, rand_core::OsRng, Error as ArgonError, PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Algorithm, Argon2, Params, Version
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::{distr::Alphanumeric, Rng};
use rocket::{futures::lock::Mutex, serde::{Deserialize, Serialize}, tokio::sync::broadcast::Sender, State};

use crate::ClientesSockets;


#[derive(Serialize,Deserialize,Clone,Debug)]
#[serde(crate="rocket::serde")]
pub struct Claims{
    pub sub:String,
    pub username:String,
    pub exp:usize
}

#[derive(Serialize,Deserialize,Clone,Debug)]
#[serde(crate="rocket::serde")]
pub struct WSNotifications{
    event:String,
}

pub struct Utils{

}

impl Utils {

    pub fn current_timestamp() -> i64 {
        SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
    }
    
    fn get_argon2d_config() -> Argon2<'static> {
        Argon2::new(
            Algorithm::Argon2d,  // <-- Usamos Argon2d específicamente
            Version::V0x13,      // Versión 1.3
            Params::new(
                65536,   // Memoria (KB) - 64MB
                3,      // Iteraciones
                4,       // Paralelismo
                None    // Tamaño de hash (None = default)
            ).unwrap()
        )
    }
    
    pub fn hash_password_argon2d(password: &str) -> Result<String, String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Self::get_argon2d_config();

        argon2.hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| format!("Error al hashear: {}", e))
    }

    pub fn verify_password_argon2d(password: &str, stored_hash: &str) -> Result<bool,ArgonError> {
        let argon2 = Self::get_argon2d_config();
        let parsed_hash = PasswordHash::new(stored_hash).ok().unwrap();
        let coincide=argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok();
        Ok(coincide)
    }

    pub async fn make_jwt(username:&str,id_usuario:&str)->Result<String,jsonwebtoken::errors::Error>{
        let key = b"1234afeb";
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let now_in_seconds = now.as_secs() as usize;

        // 7 días en segundos: 7 * 24 * 60 * 60 = 604800
        let expiration = now_in_seconds + (7 * 24 * 60 * 60);
        let claims = Claims{
            sub:id_usuario.to_string(),
            username:username.to_string(),
            exp:expiration
        };
        match encode(&Header::default(), &claims, &EncodingKey::from_secret(key)){
            Ok(res)=>{
                println!("token generado {}",res);
                Ok(res)
            },
            Err(e)=>{
                println!("err make_jwt {}",e);
                Err(e)
            }
        }
    }

    pub fn verifi_token(token:String)->Result<String,jsonwebtoken::errors::Error>{
        let key = b"1234afeb";
        match decode::<Claims>(&token,&DecodingKey::from_secret(key),&Validation::default()){
            Ok(res)=>{
                Ok(res.claims.sub)
            },
            Err(e)=>{
                println!("err verifi_token {}",e);
                Err(e)
            }
        }
    }
    pub fn generar_string(max_largo: usize) -> String {
        let largo = rand::thread_rng().gen_range(1..=max_largo);
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(largo)
            .map(char::from)
            .collect()
    }

    pub async fn send_ws(clients:&ClientesSockets,event:&str){
        let my_objects:Vec<WSNotifications>=vec![
            WSNotifications{event:event.to_string()}
        ];
        let json_string = match serde_json::to_string(&my_objects) {
            Ok(json)=>json,
            Err(e)=>{
                eprint!("error serializando JSON {:?}",e);
                return;
            }        
        };
        println!("Intentando enviar msg a todos los clientes ........");
        for (user_id,sender) in clients.lock().await.iter(){
            println!("enviando msg a {}",user_id);
            match sender.send(json_string.clone()) {
                Ok(_)=>println!("msg enviado {}",user_id)
                ,Err(e)=>println!("Err send msg {:?}",e)
                
            }
        }
    }
}