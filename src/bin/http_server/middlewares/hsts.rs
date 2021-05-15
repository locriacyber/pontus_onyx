pub struct Hsts {
	pub enable: bool,
}

impl<S> actix_web::dev::Transform<S> for Hsts
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
	type Transform = HstsMiddleware<S>;
	type Future = futures::future::Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		futures::future::ok(Self::Transform {
			service,
			enable: self.enable,
		})
	}
}

pub struct HstsMiddleware<S> {
	service: S,
	enable: bool,
}

impl<S> actix_web::dev::Service for HstsMiddleware<S>
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
		let future = self.service.call(service_request);
		let enable = self.enable;
		Box::pin(async move {
			future.await.map(|mut response| {
				if enable {
					response.response_mut().head_mut().headers.insert(
						actix_web::http::header::STRICT_TRANSPORT_SECURITY,
						actix_web::http::HeaderValue::from_str("max-age=31536000").unwrap(),
					);
				}

				response
			})
		})
	}
}
