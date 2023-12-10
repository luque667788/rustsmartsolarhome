use leptos::*;
use crate::models::User;

#[cfg(feature = "ssr")]
use axum;
#[cfg(feature = "ssr")]
use leptos_axum;
#[cfg(feature = "ssr")]
use serde_json;


#[cfg(feature = "ssr")]
pub static REMOVE_COOKIE: &str = "user=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT";

#[cfg(feature = "ssr")]
pub async fn isloged_fn(headers: &axum::http::HeaderMap) -> bool {
    //
    let user = match get_user_from_headers(headers) {
        Some(v) => { v }
        None => {
            return false;
        }
    };
    println!("{} {}",user.name,user.pass);
    
    checkcredentials(&user)
}

pub fn checkcredentials(user: &User) -> bool {
    user.name == "admin" && user.pass == "1234"
}

#[server(Login)]
pub async fn login_fn(username: String, password: String) -> Result<(), ServerFnError> {
    //
    if username == "admin" && password == "1234" {
        cookie_set_user(User {
            name: username,
            pass: password,
        }).await;
        //leptos_axum::redirect("/");
        println!("user will login");
        Ok(())
    } else {
        Err(ServerFnError::ServerError("Incorrect username or password!".into()))
    }
}

#[server(Logout)]
pub async fn logout_fn() -> Result<(), ServerFnError> {
    // remove cookies from user.
    //do not matter is is loged on or nots
    let response_options = use_context::<leptos_axum::ResponseOptions>().unwrap();
    response_options.insert_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue
            ::from_str(REMOVE_COOKIE)
            .expect("header value couldn't be set")
    );
    leptos_axum::redirect("/login");
    Ok(())
}

#[cfg(feature = "ssr")]
pub async fn cookie_set_user(user: User) {
    if let Some(res) = leptos::use_context::<leptos_axum::ResponseOptions>() {
        let payload = serde_json::to_string(&user).unwrap();
        res.insert_header(
            axum::http::header::SET_COOKIE,
            axum::http::HeaderValue
                ::from_str(&format!("user={payload}; path=/; HttpOnly"))
                .expect("header value couldn't be set")
        );
    }
}

#[server(CurrentUserAction)]
pub async fn current_user() -> Result<User, ServerFnError> {
    let Some(logged_user) = cookie_get_user() else {
        return Err(ServerFnError::ServerError("you must be logged in".into()));
    };
    
    if !checkcredentials(&logged_user){
        return Err(ServerFnError::ServerError("cookie Login credentials are incorrect".into()));
    }
    Ok(logged_user)
}

#[cfg(feature = "ssr")]
pub fn cookie_get_user() -> Option<User> {
    if let Some(req) = leptos::use_context::<leptos_axum::RequestParts>() {
        get_user_from_headers(&req.headers)
    } else {
        None
    }
}
#[cfg(feature = "ssr")]
pub fn get_user_from_headers(headers: &axum::http::HeaderMap) -> Option<User> {
    headers.get(axum::http::header::COOKIE).and_then(|x| {
        x.to_str()
            .unwrap()
            .split("; ")
            .find(|&x| x.starts_with("user"))
            .and_then(|x| x.split('=').last())
            .and_then(|x| decode_json(x).ok())
    })
}

#[cfg(feature = "ssr")]
pub fn decode_json(payload: &str) -> Result<User, serde_json::Error> {
    serde_json::from_str(payload)
}
