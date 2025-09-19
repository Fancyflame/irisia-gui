pub use implements::*;

// pub mod direct_assign_helper;
pub mod implements;
pub mod proxy_signal;

pub trait Definition {
    type Value;
    type Storage: 'static;

    fn create(&self) -> (Self::Storage, Self::Value);
    fn update(&self, storage: &mut Self::Storage);
}
