use densky_adapter::context::CloudContext;

#[derive(Debug, Default)]
#[allow(unused)]
pub struct HttpRouterContext {
    container: bool,
}

impl CloudContext for HttpRouterContext {}
