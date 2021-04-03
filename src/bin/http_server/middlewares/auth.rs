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
				// TODO : check value of the <Authorization> header
				if auth_value == "Bearer TODO" {
					let future = self.service.call(service_request);
					Box::pin(async move { future.await })
				} else {
					Box::pin(async move {
						Ok(actix_web::dev::ServiceResponse::new(
							service_request.into_parts().0,
							actix_web::HttpResponse::Unauthorized()
								.content_type("application/ld+json")
								.body(r#"{"http_code":401,"http_description":"unauthorized"}"#),
						))
					})
				}
			}
			None => {
				if service_request.path().starts_with("/public/")
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
				} else {
					Box::pin(async move {
						Ok(actix_web::dev::ServiceResponse::new(
							service_request.into_parts().0,
							actix_web::HttpResponse::Unauthorized()
								.content_type("application/ld+json")
								.body(r#"{"http_code":401,"http_description":"unauthorized"}"#),
						))
					})
				}
			}
		}
	}
}

#[actix_rt::test]
async fn hsv5femo2qgu80gbad0ov5() {
	let mut app = actix_web::test::init_service(
		actix_web::App::new()
			.wrap(super::Auth {})
			.service(crate::http_server::api::get_item)
			.service(crate::http_server::webfinger_handle),
	)
	.await;

	let tests: Vec<(&str, bool)> = vec![
		("/", true),
		("/folder/", true),
		("/document", true),
		("/webfinger", true),
		("/folder/document", true),
		("/public/folder/", true),
		("/public/document", false),
		("/public/folder/document", false),
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
