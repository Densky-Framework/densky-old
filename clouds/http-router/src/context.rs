use densky_adapter::context::CloudContext;

#[derive(Debug, Default)]
pub struct HttpRouterContext {
    container: bool,
}

impl CloudContext for HttpRouterContext {}
