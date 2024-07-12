use bevy_app::App;
use bevy_ecs::{
    event::{Event, EventReader, EventWriter},
    system::{ResMut, Resource, SystemParam},
};
use bevy_replicon::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::marker::PhantomData;

pub mod prelude {
    pub use super::{Request, RequestAppExt, RequestEvent, RequestSender, ResponseEvent};
}

pub trait Request {
    type Response;
}

#[derive(SystemParam)]
pub struct RequestSender<'w, R: Request + Send + Sync + 'static> {
    counter: ResMut<'w, RequestCounter<R>>,
    writer: EventWriter<'w, RequestEvent<R>>,
}

impl<'w, R: Request + Send + Sync> RequestSender<'w, R> {
    pub fn send(&mut self, request: R) -> usize {
        let index = self.counter.0;
        let request = RequestEvent { request, index };
        self.counter.0 += 1;
        self.writer.send(request);
        index
    }
}

#[derive(SystemParam)]
pub struct RequestHandler<'w, 's, R>
where
    R: Request + Send + Sync + 'static,
    R::Response: Send + Sync,
{
    requests: EventReader<'w, 's, FromClient<RequestEvent<R>>>,
    responses: EventWriter<'w, ToClients<ResponseEvent<R>>>,
}

impl<'w, 's, R> RequestHandler<'w, 's, R>
where
    R: Request + Clone + Send + Sync,
    R::Response: Send + Sync,
{
    pub fn handle_requests<F>(&mut self, mut f: F)
    where
        F: FnMut(&R) -> R::Response,
    {
        for FromClient { client_id, event } in self.requests.read() {
            self.responses.send(ToClients {
                mode: SendMode::Direct(*client_id),
                event: event.respond(&mut f),
            });
        }
    }
}

pub trait RequestAppExt {
    fn add_client_request<R>(&mut self)
    where
        R: Request + Serialize + DeserializeOwned + Send + Sync + 'static,
        R::Response: Serialize + DeserializeOwned + Send + Sync;
}

impl RequestAppExt for App {
    fn add_client_request<R: Request>(&mut self)
    where
        R: Serialize + DeserializeOwned + Request + Send + Sync + 'static,
        R::Response: Serialize + DeserializeOwned + Send + Sync,
    {
        self.init_resource::<RequestCounter<R>>();
        self.add_client_event::<RequestEvent<R>>(ChannelKind::Unordered);
        self.add_server_event::<ResponseEvent<R>>(ChannelKind::Unordered);
    }
}

#[derive(Resource)]
pub struct RequestCounter<R>(usize, PhantomData<R>);

impl<R> Default for RequestCounter<R> {
    fn default() -> Self {
        Self(0, PhantomData)
    }
}

#[derive(Event, Serialize, Deserialize)]
pub struct RequestEvent<R: Request> {
    pub request: R,
    index: usize,
}

impl<R: Request> RequestEvent<R> {
    pub fn respond<F>(&self, f: F) -> ResponseEvent<R>
    where
        F: FnOnce(&R) -> R::Response,
    {
        ResponseEvent {
            response: f(&self.request),
            index: self.index,
        }
    }
}

#[derive(Event, Serialize, Deserialize)]
pub struct ResponseEvent<R: Request> {
    pub response: R::Response,
    index: usize,
}

impl<R: Request> ResponseEvent<R> {
    pub fn index(&self) -> usize {
        self.index
    }
}
