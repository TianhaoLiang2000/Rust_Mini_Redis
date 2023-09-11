#![feature(impl_trait_in_assoc_type)]

use std::{collections::HashMap, ops::Deref, sync::Mutex};

use anyhow::Ok;
use lazy_static::lazy_static;
pub struct S;

lazy_static! {
    static ref MY_MAP: Mutex<HashMap<i64, volo_gen::volo::example::Item>> =
        Mutex::new(HashMap::new());
}

#[volo::async_trait]
impl volo_gen::volo::example::ItemService for S {
    async fn get_item(
        &self,
        _req: volo_gen::volo::example::GetItemRequest,
    ) -> core::result::Result<volo_gen::volo::example::GetItemResponse, volo_thrift::AnyhowError>
    {
        let my_map_lock = MY_MAP.lock().unwrap();
        let my_map = my_map_lock.deref();
        let item_value = my_map.get(&_req.id);
        let item = item_value.unwrap();
        Ok(volo_gen::volo::example::GetItemResponse { item: item.clone() })
    }

    async fn set_item(
        &self,
        _req: volo_gen::volo::example::SetItemRequest,
    ) -> core::result::Result<volo_gen::volo::example::SetItemResponse, volo_thrift::AnyhowError>
    {
        let item = volo_gen::volo::example::Item {
            id: _req.id,
            title: _req.title,
            content: _req.content,
            extra: Some(std::collections::HashMap::new()),
        };
        {
            let mut my_map = MY_MAP.lock().unwrap();
            my_map.insert(_req.id, item.clone());
        }
        Ok(volo_gen::volo::example::SetItemResponse { item })
    }

    async fn del_item(
        &self,
        _req: volo_gen::volo::example::DelItemRequest,
    ) -> core::result::Result<volo_gen::volo::example::DelItemResponse, volo_thrift::AnyhowError>
    {
        #[warn(unused_assignments)]
        let del: bool;
        match MY_MAP.lock().unwrap().remove(&_req.id) {
            Some(_item) => del = true,
            None => del = false,
        }

        Ok(volo_gen::volo::example::DelItemResponse { del })
    }

    async fn ping(
        &self,
        _req: volo_gen::volo::example::PingRequest,
    ) -> core::result::Result<volo_gen::volo::example::PingResponse, volo_thrift::AnyhowError> {
        Ok(volo_gen::volo::example::PingResponse { ping: true })
    }
}

#[derive(Clone)]
pub struct LogService<S>(S);

#[volo::service]
impl<Cx, Req, S> volo::Service<Cx, Req> for LogService<S>
where
    Req: std::fmt::Debug + Send + 'static,
    S: Send + 'static + volo::Service<Cx, Req> + Sync,
    S::Response: std::fmt::Debug,
    S::Error: std::fmt::Debug,
    Cx: Send + 'static,
{
    async fn call(&self, cx: &mut Cx, req: Req) -> Result<S::Response, S::Error> {
        let now = std::time::Instant::now();
        tracing::debug!("Received request {:?}", &req);
        let resp = self.0.call(cx, req).await;
        tracing::debug!("Sent response {:?}", &resp);
        tracing::info!("Request took {}ms", now.elapsed().as_millis());
        resp
    }
}

pub struct LogLayer;

impl<S> volo::Layer<S> for LogLayer {
    type Service = LogService<S>;

    fn layer(self, inner: S) -> Self::Service {
        LogService(inner)
    }
}
