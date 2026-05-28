use actix_session::Session;
use actix_web::HttpResponse;

pub fn is_logged_in(session: &Session) -> bool {
    get_user_id(session).is_some()
}

pub fn get_user_id(session: &Session) -> Option<i32> {
    session.get::<i32>("user_id").unwrap_or(None)
}

pub fn redirect_to_login() -> HttpResponse {
    HttpResponse::Found()
        .append_header(("Location", "/login"))
        .finish()
}
