use super::WalkerEntity;

#[derive(Debug, Clone)]
pub struct WalkerThorn {
    id: usize,
}

impl WalkerEntity for WalkerThorn {
    fn get_id(&self) -> usize {
        self.id
    }

    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
}
