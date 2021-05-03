use rand::seq::IteratorRandom;
use rand::Rng;

pub struct Auth;

impl<S> actix_web::dev::Transform<S> for Auth
where
	S: actix_web::dev::Service<
		Request = actix_web::dev::ServiceRequest,
		Response = actix_web::dev::ServiceResponse<actix_web::dev::Body>,
		Error = actix_web::Error,
	>,
	S::Future: 'static,
{
	type Request = actix_web::dev::ServiceRequest;
	type Response = actix_web::dev::ServiceResponse<actix_web::dev::Body>;
	type Error = actix_web::Error;
	type InitError = ();
	type Transform = AuthMiddleware<S>;
	type Future = futures::future::Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		futures::future::ok(Self::Transform { service })
	}
}

pub struct AuthMiddleware<S> {
	service: S,
}

impl<S> actix_web::dev::Service for AuthMiddleware<S>
where
	S: actix_web::dev::Service<
		Request = actix_web::dev::ServiceRequest,
		Response = actix_web::dev::ServiceResponse<actix_web::dev::Body>,
		Error = actix_web::Error,
	>,
	S::Future: 'static,
{
	type Request = actix_web::dev::ServiceRequest;
	type Response = actix_web::dev::ServiceResponse<actix_web::dev::Body>;
	type Error = actix_web::Error;
	type Future =
		std::pin::Pin<Box<dyn futures::Future<Output = Result<Self::Response, Self::Error>>>>;

	fn poll_ready(
		&mut self,
		ctx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), Self::Error>> {
		self.service.poll_ready(ctx)
	}

	fn call(&mut self, service_request: Self::Request) -> Self::Future {
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

				match tokens.iter().find(|e| e.name() == search_token) {
					Some(token) => {
						// TODO : check token validity
						// TODO : check token scopes
						let future = self.service.call(service_request);
						Box::pin(async move { future.await })
					}
					None => Box::pin(async move {
						Ok(actix_web::dev::ServiceResponse::new(
							service_request.into_parts().0,
							super::super::api::build_response(
								actix_web::http::StatusCode::UNAUTHORIZED,
								None,
								None,
								true,
							),
						))
					}),
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
				} else if service_request.method() == actix_web::http::Method::OPTIONS {
					let future = self.service.call(service_request);
					Box::pin(async move { future.await })
				} else {
					Box::pin(async move {
						Ok(actix_web::dev::ServiceResponse::new(
							service_request.into_parts().0,
							super::super::api::build_response(
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

#[derive(Clone, Debug)]
pub struct OauthFormToken {
	ip: std::net::SocketAddr,
	forged: std::time::Instant,
	value: String,
}
impl OauthFormToken {
	pub fn new(ip: std::net::SocketAddr) -> Self {
		let forged = std::time::Instant::now();
		let mut value = String::new();

		let mut rng_limit = rand::thread_rng();
		for _ in 1..rng_limit.gen_range(32, 64) {
			let mut rng_item = rand::thread_rng();
			value.push(
				crate::FORM_TOKEN_ALPHABET
					.chars()
					.choose(&mut rng_item)
					.unwrap(),
			);
		}

		Self { ip, forged, value }
	}
}
impl OauthFormToken {
	pub fn ip(&self) -> std::net::SocketAddr {
		return self.ip;
	}
	pub fn forged(&self) -> std::time::Instant {
		return self.forged;
	}
	pub fn value(&self) -> String {
		return self.value.clone();
	}
	pub fn has_expirated(&self) -> bool {
		(std::time::Instant::now() - self.forged) == std::time::Duration::from_secs(5 * 60)
	}
	pub fn should_be_cleaned(&self, ip: &std::net::SocketAddr) -> bool {
		if self.has_expirated() {
			return true;
		}

		if &self.ip == ip {
			return true;
		}

		return false;
	}
}

#[actix_rt::test]
async fn hsv5femo2qgu80gbad0ov5() {
	let mut app = actix_web::test::init_service(
		actix_web::App::new()
			.wrap(super::Auth {})
			.service(crate::http_server::favicon)
			.service(crate::http_server::api::get_item)
			.service(crate::http_server::webfinger_handle)
			.service(crate::http_server::get_oauth)
			.service(crate::http_server::post_oauth),
	)
	.await;

	let tests: Vec<(&str, bool)> = vec![
		("/storage/", true),
		("/storage/folder/", true),
		("/storage/document", true),
		("/storage/folder/document", true),
		("/storage/public/folder/", true),
		("/.well-known/webfinger", false),
		("/storage/public/document", false),
		("/storage/public/folder/document", false),
		("/oauth", false),
		("/favicon.ico", false),
	];

	for test in tests {
		println!("request to {}", test.0);

		let request = actix_web::test::TestRequest::get().uri(test.0).to_request();
		let response = actix_web::test::call_service(&mut app, request).await;

		if test.1 {
			assert_eq!(response.status(), actix_web::http::StatusCode::UNAUTHORIZED);
		} else {
			assert_ne!(response.status(), actix_web::http::StatusCode::UNAUTHORIZED);
		}
	}
}
