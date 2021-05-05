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

#[actix_web::get("/remotestorage.svg")]
pub async fn remotestoragesvg() -> actix_web::web::HttpResponse {
	return actix_web::HttpResponse::Ok().body(actix_web::web::Bytes::from_static(include_bytes!(
		"static/remotestorage.svg"
	)));
}

#[actix_web::get("/")]
pub async fn index() -> actix_web::web::HttpResponse {
	actix_web::HttpResponse::Ok()
		.body(format!(r#"<!DOCTYPE html>
<html>
	<head>
		<meta charset="UTF-8">
		<meta http-equiv="X-UA-Compatible" content="IE=edge">
		<meta name="viewport" content="width=device-width, initial-scale=1.0">
		<title>{}</title>
	</head>
	<body style="padding:1em 2em;">
		<h1><img src="/favicon.ico" alt="" style="max-height:2em;vertical-align:middle;"> {}</h1>
		<p>This is an <a href="https://remotestorage.io/"><img src="/remotestorage.svg" style="max-height:1em;vertical-align:middle;"> remoteStorage</a> server.</p>
		<p><a href="https://wiki.remotestorage.io/Apps">Find Apps compatible</a> with this database.</p>
		<p>See source code on <a href="https://github.com/Jimskapt/pontus_onyx">GitHub</a>.</p>
	</body>
</html>"#, env!("CARGO_PKG_NAME"), env!("CARGO_PKG_NAME")))
}
