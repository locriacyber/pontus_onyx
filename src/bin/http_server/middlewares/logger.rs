use std::sync::{Arc, Mutex};

pub struct Logger {
	pub logger: Arc<Mutex<charlie_buffalo::Logger>>,
}

impl<S> actix_web::dev::Transform<S> for Logger
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
	type Transform = LoggerMiddleware<S>;
	type Future = futures::future::Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		futures::future::ok(Self::Transform {
			service,
			logger: self.logger.clone(),
		})
	}
}

pub struct LoggerMiddleware<S> {
	service: S,
	logger: Arc<Mutex<charlie_buffalo::Logger>>,
}

type OutputFuture = std::pin::Pin<
	Box<
		dyn futures::Future<
			Output = Result<
				actix_web::dev::ServiceResponse<actix_web::dev::Body>,
				actix_web::Error,
			>,
		>,
	>,
>;

impl<S> actix_web::dev::Service for LoggerMiddleware<S>
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
	type Future = OutputFuture;

	fn poll_ready(
		&mut self,
		ctx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), Self::Error>> {
		self.service.poll_ready(ctx)
	}

	fn call(&mut self, service_request: Self::Request) -> Self::Future {
		let mut attributes = vec![
			(String::from("event"), String::from("http_access")),
			(
				String::from("method"),
				format!("{}", service_request.method()),
			),
			(String::from("path"), String::from(service_request.path())),
			(
				String::from("query"),
				String::from(service_request.query_string()),
			),
		];
		if let Some(peer_addr) = service_request.connection_info().realip_remote_addr() {
			attributes.push((String::from("client_ip"), format!("{}", peer_addr)));
		}
		for (header_name, header_value) in service_request.headers() {
			attributes.push((
				format!("request_header:{}", header_name),
				String::from(header_value.to_str().unwrap_or_default()),
			));
		}
		attributes.push((
			String::from("protocol"),
			String::from(service_request.uri().scheme_str().unwrap_or("http?")),
		));

		let future = self.service.call(service_request);

		let logger_for_response = self.logger.clone();
		Box::pin(async move {
			let res = future.await;

			if let Ok(response) = &res {
				attributes.push((
					String::from("response_code"),
					String::from(response.status().as_str()),
				));

				for (header_name, header_value) in response.headers() {
					attributes.push((
						format!("response_header:{}", header_name),
						String::from(header_value.to_str().unwrap_or_default()),
					));
				}
			}

			charlie_buffalo::push(&logger_for_response, attributes, None);

			res
		})
	}
}
