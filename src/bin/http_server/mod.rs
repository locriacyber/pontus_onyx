mod api;
mod middlewares;
mod tokens;
mod webfinger;

pub use api::*;
pub use middlewares::*;
pub use tokens::*;
pub use webfinger::webfinger_handle;

#[actix_web::get("/favicon.ico")]
pub async fn favicon() -> actix_web::web::HttpResponse {
	return actix_web::HttpResponse::Ok().body(actix_web::web::Bytes::from_static(include_bytes!(
		"static/favicon.ico"
	)));
}
