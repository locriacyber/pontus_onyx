use std::sync::{Arc, Mutex};

#[cfg(test)]
mod tests;
mod token;

pub use token::*;

pub struct Auth {
	pub logger: Arc<Mutex<charlie_buffalo::Logger>>,
}

impl<S> actix_web::dev::Transform<S, actix_web::dev::ServiceRequest> for Auth
where
	S: actix_web::dev::Service<
		actix_web::dev::ServiceRequest,
		Response = actix_web::dev::ServiceResponse<actix_web::body::BoxBody>,
		Error = actix_web::Error,
	>,
	S::Future: 'static,
{
	type Response = actix_web::dev::ServiceResponse<actix_web::body::BoxBody>;
	type Error = actix_web::Error;
	type InitError = ();
	type Transform = AuthMiddleware<S>;
	type Future = futures::future::Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		futures::future::ok(Self::Transform {
			service,
			logger: self.logger.clone(),
		})
	}
}

pub struct AuthMiddleware<S> {
	service: S,
	logger: Arc<Mutex<charlie_buffalo::Logger>>,
}

impl<S> actix_web::dev::Service<actix_web::dev::ServiceRequest> for AuthMiddleware<S>
where
	S: actix_web::dev::Service<
		actix_web::dev::ServiceRequest,
		Response = actix_web::dev::ServiceResponse<actix_web::body::BoxBody>,
		Error = actix_web::Error,
	>,
	S::Future: 'static,
{
	type Response = actix_web::dev::ServiceResponse<actix_web::body::BoxBody>;
	type Error = actix_web::Error;
	type Future =
		futures_util::future::LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

	actix_web::dev::forward_ready!(service);

	fn call(&self, service_request: actix_web::dev::ServiceRequest) -> Self::Future {
		let request_method = service_request.method().clone();

		match service_request.head().headers().get("Authorization") {
			Some(auth_value) => {
				let search_token = auth_value
					.to_str()
					.unwrap()
					.strip_prefix("Bearer ")
					.unwrap()
					.trim();
				let tokens = service_request
					.app_data::<actix_web::web::Data<
						std::sync::Arc<std::sync::Mutex<Vec<crate::http_server::AccessBearer>>>,
					>>()
					.unwrap()
					.lock()
					.unwrap()
					.clone();

				let settings = service_request
					.app_data::<actix_web::web::Data<
						std::sync::Arc<std::sync::Mutex<crate::http_server::Settings>>,
					>>()
					.unwrap()
					.lock()
					.unwrap()
					.clone();

				match tokens.iter().find(|e| e.get_name() == search_token) {
					Some(token) => {
						// TODO : check token validity with client_id
						// TODO : test access in case of module = "*"
						// TODO : test token expiration

						if (std::time::Instant::now() - *token.get_emit_time())
							< std::time::Duration::from_secs(
								settings.token_lifetime_seconds.unwrap_or_else(|| {
									crate::http_server::Settings::default()
										.token_lifetime_seconds
										.unwrap()
								}),
							) {
							match token.get_scopes().iter().find(|scope| {
								if scope
									.allowed_methods()
									.iter()
									.any(|method| method == service_request.method())
								{
									if scope.module == "*" {
										service_request.path().starts_with("/storage/")
									} else {
										service_request.path().starts_with(&format!(
											"/storage/{}/{}",
											token.get_username(),
											scope.module
										)) || service_request.path().starts_with(&format!(
											"/storage/public/{}/{}",
											token.get_username(),
											scope.module
										))
									}
								} else {
									false
								}
							}) {
								Some(_) => {
									let future = self.service.call(service_request);
									Box::pin(async move { future.await })
								}
								None => {
									self.logger.lock().unwrap().push(
										vec![
											(String::from("event"), String::from("auth")),
											(String::from("token"), String::from(search_token)),
											(String::from("level"), String::from("DEBUG")),
										],
										Some("unauthorized scope"),
									);

									Box::pin(async move {
										// TODO : check security issue about this ?
										let all_origins =
											actix_web::http::header::HeaderValue::from_bytes(b"*")
												.unwrap();
										let headers = service_request.headers().clone();
										let origin = headers
											.get(actix_web::http::header::ORIGIN)
											.unwrap_or(&all_origins)
											.to_str()
											.unwrap();

										Ok(actix_web::dev::ServiceResponse::new(
											service_request.into_parts().0,
											pontus_onyx::database::build_http_json_response(
												origin,
												&request_method,
												actix_web::http::StatusCode::FORBIDDEN,
												None,
												None,
												true,
											),
										))
									})
								}
							}
						} else {
							self.logger.lock().unwrap().push(
								vec![
									(String::from("event"), String::from("auth")),
									(String::from("token"), String::from(search_token)),
									(String::from("level"), String::from("DEBUG")),
								],
								Some("expirated token"),
							);

							Box::pin(async move {
								// TODO : check security issue about this ?
								let all_origins =
									actix_web::http::header::HeaderValue::from_bytes(b"*").unwrap();
								let headers = service_request.headers().clone();
								let origin = headers
									.get(actix_web::http::header::ORIGIN)
									.unwrap_or(&all_origins)
									.to_str()
									.unwrap();

								Ok(actix_web::dev::ServiceResponse::new(
									service_request.into_parts().0,
									pontus_onyx::database::build_http_json_response(
										origin,
										&request_method,
										actix_web::http::StatusCode::FORBIDDEN,
										None,
										None,
										true,
									),
								))
							})
						}
					}
					None => {
						Box::pin(async move {
							// TODO : check security issue about this ?
							let all_origins =
								actix_web::http::header::HeaderValue::from_bytes(b"*").unwrap();
							let headers = service_request.headers().clone();
							let origin = headers
								.get(actix_web::http::header::ORIGIN)
								.unwrap_or(&all_origins)
								.to_str()
								.unwrap();

							Ok(actix_web::dev::ServiceResponse::new(
								service_request.into_parts().0,
								pontus_onyx::database::build_http_json_response(
									origin,
									&request_method,
									actix_web::http::StatusCode::UNAUTHORIZED,
									None,
									None,
									true,
								),
							))
						})
					}
				}
			}
			None => {
				if service_request.path().starts_with("/storage/public/")
					&& service_request
						.path()
						.split('/')
						.collect::<Vec<&str>>()
						.last()
						.unwrap_or(&"") != &""
					&& (service_request.method() == actix_web::http::Method::GET
						|| service_request.method() == actix_web::http::Method::HEAD
						|| service_request.method() == actix_web::http::Method::OPTIONS)
				{
					let future = self.service.call(service_request);
					Box::pin(async move { future.await })
				} else if service_request.path().starts_with("/.well-known/webfinger")
					&& (service_request.method() == actix_web::http::Method::GET
						|| service_request.method() == actix_web::http::Method::HEAD
						|| service_request.method() == actix_web::http::Method::OPTIONS)
				{
					let future = self.service.call(service_request);
					Box::pin(async move { future.await })
				} else if service_request.path().starts_with("/oauth")
					&& (service_request.method() == actix_web::http::Method::GET
						|| service_request.method() == actix_web::http::Method::POST)
				{
					let future = self.service.call(service_request);
					Box::pin(async move { future.await })
				} else if service_request.path() == "/favicon.ico"
					&& service_request.method() == actix_web::http::Method::GET
				{
					let future = self.service.call(service_request);
					Box::pin(async move { future.await })
				} else if service_request.path() == "/remotestorage.svg"
					&& service_request.method() == actix_web::http::Method::GET
				{
					let future = self.service.call(service_request);
					Box::pin(async move { future.await })
				} else if service_request.path() == "/"
					&& service_request.method() == actix_web::http::Method::GET
				{
					let future = self.service.call(service_request);
					Box::pin(async move { future.await })
				} else if service_request.method() == actix_web::http::Method::OPTIONS {
					let future = self.service.call(service_request);
					Box::pin(async move { future.await })
				} else {
					Box::pin(async move {
						// TODO : check security issue about this ?
						let all_origins =
							actix_web::http::header::HeaderValue::from_bytes(b"*").unwrap();
						let headers = service_request.headers().clone();
						let origin = headers
							.get(actix_web::http::header::ORIGIN)
							.unwrap_or(&all_origins)
							.to_str()
							.unwrap();

						Ok(actix_web::dev::ServiceResponse::new(
							service_request.into_parts().0,
							pontus_onyx::database::build_http_json_response(
								origin,
								&request_method,
								actix_web::http::StatusCode::UNAUTHORIZED,
								None,
								None,
								true,
							),
						))
					})
				}
			}
		}
	}
}
