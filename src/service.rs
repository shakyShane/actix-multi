use std::path::{PathBuf};
use std::task::{Context, Poll};
use futures::future::{ready, Ready, LocalBoxFuture, ok};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse, HttpServiceFactory, AppService, ResourceDef},
    error::Error,
    HttpResponse
};
use actix_service::{Service, ServiceFactory};
use futures::FutureExt;
use crate::files::Files;

pub trait MultiServiceTrait: Service<
    Request = ServiceRequest,
    Response = ServiceResponse,
    Error = Error,
    Future = Ready<Result<ServiceResponse, Error>>,
> {
    fn check_multi(&self, req: &ServiceRequest) -> bool;
}

pub struct Multi {
    pub items: Vec<Files>
}

impl Multi {
    pub fn new(items: Vec<Files>) -> Self {
        Self { items }
    }
}

impl ServiceFactory for Multi {
    type Request = ServiceRequest;
    type Response = ServiceResponse;
    type Error = Error;
    type Config = ();
    type Service = MultiService;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<Self::Service, Self::InitError>>;

    fn new_service(&self, _: ()) -> Self::Future {
        let srv = MultiService { items: self.items.clone() };
        ok(srv).boxed_local()
    }
}

impl HttpServiceFactory for Multi {
    fn register(self, config: &mut AppService) {
        let rdef = ResourceDef::root_prefix("/");
        config.register_service(rdef, None, self, None)
    }
}

pub struct MultiService {
    pub items: Vec<Files>
}

impl Service for MultiService {
    type Request = ServiceRequest;
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Ready<Result<ServiceResponse, Error>>;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        let mut handles: Vec<Files> = self.items.iter()
            .filter(|srv| srv.check_multi(&req))
            .map(|srv| srv.to_owned())
            .collect();

        if let Some(mut h) = handles.get_mut(0) {
            println!("using {:?}", h);
            h.call(req)
        } else {
            let resp = HttpResponse::NotFound().finish();
            let (req, _) = req.into_parts();
            let srv_resp = ServiceResponse::new(req, resp);
            ready(Ok(srv_resp))
        }
    }
}
