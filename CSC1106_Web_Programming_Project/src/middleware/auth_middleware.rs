use std::future::{ready, Ready};
use std::rc::Rc;

use actix_session::SessionExt;
use actix_web::body::EitherBody;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpResponse};

pub struct RequireAuth;

impl RequireAuth {
	pub fn new() -> Self {
		Self
	}
}

fn is_public_path(path: &str) -> bool {
	matches!(path, "/" | "/login" | "/register" | "/logout") || path.starts_with("/static/")
}

impl<S, B> Transform<S, ServiceRequest> for RequireAuth
where
	S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
	B: 'static,
{
	type Response = ServiceResponse<EitherBody<B>>;
	type Error = Error;
	type InitError = ();
	type Transform = RequireAuthMiddleware<S>;
	type Future = Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		ready(Ok(RequireAuthMiddleware {
			service: Rc::new(service),
		}))
	}
}

pub struct RequireAuthMiddleware<S> {
	service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequireAuthMiddleware<S>
where
	S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
	B: 'static,
{
	type Response = ServiceResponse<EitherBody<B>>;
	type Error = Error;
	type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

	forward_ready!(service);

	fn call(&self, req: ServiceRequest) -> Self::Future {
		let service = Rc::clone(&self.service);
		let path = req.path().to_string();

		Box::pin(async move {
			if is_public_path(&path) {
				return service.call(req).await.map(|response| response.map_into_left_body());
			}

			let is_logged_in = req
				.get_session()
				.get::<i32>("user_id")
				.unwrap_or(None)
				.is_some();

			if is_logged_in {
				return service.call(req).await.map(|response| response.map_into_left_body());
			}

			let response = HttpResponse::Found()
				.append_header(("Location", "/login"))
				.finish()
				.map_into_right_body();

			Ok(req.into_response(response))
		})
	}
}
