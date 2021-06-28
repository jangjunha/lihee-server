use async_trait::async_trait;

pub use crate::Book;

#[async_trait]
pub trait DataSource {
    async fn search(
        &self,
        keyword: &str,
    ) -> Result<Vec<Book>, Box<dyn std::error::Error + Send + Sync>>;
}
