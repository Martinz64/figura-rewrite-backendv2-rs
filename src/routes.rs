use async_std::fs;
use async_std::fs::File;
use async_std::io;
use async_std::io::BufWriter;
use tide::Body;
use tide::Request;
use tide::Response;
use tide::StatusCode;
use tide::prelude::*;
use serde_json::Value;
use uuid::Uuid;

use crate::objects::UserInfo;
use crate::state::State;
use crate::utils::calculate_file_sha256;
use crate::utils::format_uuid;
use crate::utils::generate_hex_string;
use crate::objects::{Stage1LoginParams,Stage2LoginParams};

pub async fn debug_info(req: Request<State>) -> tide::Result<impl Into<Response>> {
    let state = req.state();

    let shared_server_ids = state.server_ids.clone();

    let server_ids = shared_server_ids.lock().await;
    let tokens = state.tokens.lock().await;

    let mut s = String::new();
    
    s.push_str("Server id to username:\n");
    server_ids.iter().for_each(|(sid,username)|{
        log::info!("sid: {}, {}",sid,username);
        s.push_str(&format!("{} → {}",sid,username));
        s.push('\n');
    });

    s.push_str("Assigned tokens:\n");
    tokens.iter().for_each(|(token,userinfo)|{
        s.push_str(&format!("{} → {} ({})",token,userinfo.username,userinfo.uuid));
        s.push('\n');
    });

    s.push_str("End\n");

    Ok(s)
}


pub async fn login_stage1(req: Request<State>) -> tide::Result<impl Into<Response>> {
    let server_id = generate_hex_string(32);

    let loginparams: Stage1LoginParams = req.query()?;
    //println!("{}",server_id).await;

    let username = loginparams.username;

    let mut server_ids = req.state().server_ids.lock().await;


    log::info!("Giving server id {} to {}",server_id.clone(),username.clone());
    
    server_ids.insert(server_id.clone(),username);
    Ok(format!("{}", server_id ))
}

pub async fn login_stage2(req: Request<State>) -> tide::Result<impl Into<Response>> {
    let token = generate_hex_string(16);

    let loginparams: Stage2LoginParams = req.query()?;

    let server_ids = req.state().server_ids.lock().await;
    let mut tokens: async_std::sync::MutexGuard<'_, std::collections::HashMap<String, UserInfo>> = req.state().tokens.lock().await;

    let auth_server_id = loginparams.id;

    if let Some(auth_username) = server_ids.get(&auth_server_id) {
        
        let request_url = format!("https://sessionserver.mojang.com/session/minecraft/hasJoined?username={}&serverId={}",auth_username,auth_server_id);

        let mut response = req
            .state()
            .http_client
            .get(request_url)
            .send()
            .await
            .map_err(|_| tide::Error::from_str(StatusCode::InternalServerError, "Failed to send request"))?;

        
        if !response.status().is_success() {
            return Err(tide::Error::from_str(
                StatusCode::InternalServerError,
                "Failed to retrieve data from the external API",
            ));
        }

        let response_body = response
            .body_string()
            .await
            .map_err(|_| tide::Error::from_str(StatusCode::InternalServerError, "Failed to read response body as string"))?;

        // Deserialize the response body as JSON
        let response_json: Value = serde_json::from_str(&response_body)
            .map_err(|_| tide::Error::from_str(StatusCode::InternalServerError, "Failed to parse response body as JSON"))?;

        let result_name = response_json["name"].as_str().unwrap();
        let result_uuid = response_json["id"].as_str().unwrap();
        
        log::info!("{}: {}",result_name, format_uuid(result_uuid.to_string()).unwrap());
        
        let userinfo = UserInfo{
            uuid: Uuid::parse_str(&result_uuid)?,
            username: result_name.to_string()
        }; 

        tokens.insert(token.clone(), userinfo);

        Ok(token)
    } else {
        Err(tide::Error::from_str(
            StatusCode::NotFound,
            "Failed to authenticate",
        ))
    }

    
}

pub async fn version(_req: Request<State>) -> tide::Result<impl Into<Response>> {
    Ok(json!({
        "release":"0.1.69",
        "prerelease":"0.1.69"
    }))
}

pub async fn limits(_req: Request<State>) -> tide::Result<impl Into<Response>> {
    Ok(json!({
        "rate": {
          "pingSize": 1024,
          "pingRate": 32,
          "equip": 1,
          "download": 50,
          "upload": 1
        },
        "limits": {
          "maxAvatarSize": 100000,
          "maxAvatars": 10,
          "allowedBadges": {
            "special": [0,0,0,0,0,0],
            "pride": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
          }
        }
    }))
}

pub async fn user_info(req: Request<State>) -> tide::Result<impl Into<Response>> {
    let uuid = req.param("uuid")?;
    log::error!("getting info for {}",uuid);

    let formatted_uuid = format_uuid(uuid.to_string()).unwrap();

    let avatar_file = format!("avatars/{}.moon",formatted_uuid);

    let mut user_info_response = json!({
        "uuid": formatted_uuid,
        "rank": "normal",
        "equipped": [],
        "lastUsed": "2023-06-14T07:16:21.265Z",
        "equippedBadges": {
            "special": [0,0,0,0,0,0],
            "pride": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
        },
        "version": "0.1.0-rc.14+1.19.4-b414b91c",
        "banned": false
    });

    if fs::metadata(&avatar_file).await.is_ok() {
        if let Some(equipped) = user_info_response.get_mut("equipped").and_then(Value::as_array_mut){
            match calculate_file_sha256(&avatar_file){
                Ok(hash) => {
                    equipped.push(json!({
                        "id": "avatar",
                        "owner": formatted_uuid,
                        "hash": hash
                    }))
                }
                Err(_e) => {}
            }

            
        }
    }

    Ok(user_info_response.to_string())

}

pub async fn download_avatar(req: Request<State>) -> tide::Result {
    let uuid = req.param("uuid")?;
    log::info!("Requested avatar for {}",uuid);
    //match Body::from_file("avatars/74cf2ba3-f346-4dfe-b3b5-f453b9f5cc5e.moon").await  {
    match Body::from_file(format!("avatars/{}.moon",uuid)).await  {
        Ok(body) => Ok(Response::builder(StatusCode::Ok).body(body).build()),
        Err(e) => Err(e.into()),
    }
}

pub async fn upload_avatar(mut req: Request<State>) -> tide::Result<impl Into<Response>>  {
    
    let mut request_data = req.take_body();

    let token = req.header("token").unwrap();
    let userinfos = req.state().tokens.lock().await;

    if let Some(user_info) = userinfos.get(token.as_str()) {
        log::info!("{} ({}) tried to upload",user_info.username,user_info.uuid);
        let avatar_file = format!("avatars/{}.moon",user_info.uuid);
        let mut file = BufWriter::new(File::create(&avatar_file).await?);
        io::copy(&mut request_data, &mut file).await?;
    }

    Ok("success")
}

pub async fn equip_avatar(_req: Request<State>) -> tide::Result<impl Into<Response>>  {
    Ok("success")
}