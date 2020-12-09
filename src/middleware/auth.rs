use hyper::{Request, Response, Body};
use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;
use std::sync::Arc;
use base64::decode as base64_decode;
use jsonwebtoken as jwt;
use serde::{Serialize, Deserialize};
use crate::config::*;
use crate::middleware::{MwPreRequest, Middleware, MiddlewareRequest};



#[derive(Debug)]
pub struct AuthMiddleware {
    clients: Arc<HashMap<String, ClientId>>,
    service_auth: HashMap<String, AuthSetting>,
}

impl Default for AuthMiddleware {
    fn default() -> Self {
        AuthMiddleware { clients: Arc::new(HashMap::new()), service_auth: HashMap::new() }
    }
}


impl Middleware for AuthMiddleware {


    fn work(&mut self, task: MiddlewareRequest) -> Pin<Box<dyn Future<Output=()> + Send>> {
        match task {
            MiddlewareRequest::Request(MwPreRequest{mut context,  request, result}) => {
                let auth_type = self.service_auth.get(&context.service_id).unwrap().clone();
                let clients = Arc::clone(&self.clients);
                tokio::spawn(async move {
                    let client_id = verify_token(&request, auth_type, clients).await;
                    match client_id {
                        Some(cid) => {
                            context.client.replace(cid);
                            result.send(Ok((request, context))).unwrap();
                        },
                        None => {
                            let err = Response::new(Body::from("auth failed"));
                            result.send(Err(err)).unwrap();
                        },
                    }
                });
            },
            MiddlewareRequest::Response(resp) => {
                resp.result.send(resp.response).unwrap();
            },
        }
        Box::pin(async {})
    }

    fn config_update(&mut self, update: crate::config::ConfigUpdate) {
        todo!()
    }

}

async fn verify_token(request: &Request<Body>, auth_type: AuthSetting, clients: Arc<HashMap<String, ClientId>>) -> Option<ClientId> {
    let client_id: Option<&ClientId> = match auth_type {
        AuthSetting::AppKey(AppKeyAuth { header_name: _header, param_name: _param }) => {
            let token = get_auth_token(request);
            clients.get(&token)
        },
        AuthSetting::Basic(BasicAuth {}) => {
            let token = get_auth_token(request);
            let ts = base64_decode(&token).ok()
                    .map(|s| String::from_utf8(s).unwrap_or(String::from(":")))
                    .unwrap_or(String::from(":"));
            let segs: Vec<&str> = ts.split(':').collect();
            let key = *segs.get(0)?;
            let secret = *segs.get(1)?;
            let client_id = clients.get(key)
                .filter(|cid| cid.app_secret.eq(secret));
            client_id
        },
        AuthSetting::JWT(JwtAuth {identity: _sub}) => {
            let token = get_auth_token(request);
            let t = jwt::dangerous_insecure_decode::<JwtClaims>(&token).ok()?;
            let app_key = &t.claims.sub;
            let app_secret = {
                clients.get(app_key).map(|c| c.app_secret.clone())
            };
            if let Some(secret) = app_secret {
                let v = jwt::Validation::new(jwt::Algorithm::HS256);
                let st = jwt::decode::<JwtClaims>(&token, &jwt::DecodingKey::from_secret(secret.as_bytes()), &v);
                let client_id = st.ok()
                    .filter(|r| r.claims.iss.eq("APIHUB"))
                    .map(|r| r.claims.sub)
                    .map(|k| clients.get(&k))
                    .flatten();
                client_id
            } else {
                None
            }
        },
        AuthSetting::OAuth2(OAuth2Auth {token_verify_url: _url}) => {
            None
        },
    };
    match client_id {
        Some(cid) => Some(cid.clone()),
        None => None,
    }
}

fn get_auth_token(req: &Request<Body>) -> String {
    let headers = req.headers();
    if let Some(token) = headers.get(hyper::header::AUTHORIZATION) {  // find in authorization header
        let segs: Vec<&str> = token.to_str().unwrap().split(' ').collect();
        String::from(*(segs.get(1).unwrap_or(&"")))
    } else {
        String::from("")
    }
}


#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    pub aud: Option<String>,         // Optional. Audience
    pub exp: usize,                  // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    pub iat: Option<usize>,          // Optional. Issued at (as UTC timestamp)
    pub iss: String,                 // Optional. Issuer
    pub nbf: Option<usize>,          // Optional. Not Before (as UTC timestamp)
    pub sub: String,                 // Optional. Subject (whom token refers to)
}
