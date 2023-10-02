pub trait WalkerEntity {
    fn get_id(&self) -> usize;
    fn set_id(&mut self, id: usize);
}
