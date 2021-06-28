mod config;
mod datasource;

pub mod lihee {
    tonic::include_proto!("heek.lihee");
}
use config::Config;
use datasource::{elasticsearch::Elasticsearch, DataSource};
use elasticsearch::{http::transport::Transport as ESTransport, Elasticsearch as ESClient};
use lihee::search_server::{Search, SearchServer};
use lihee::GetBooksPayload;
pub use lihee::{Book, Library};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

struct SearchService {
    datasources: Arc<Vec<Box<dyn DataSource + Send + Sync>>>, // TODO: DataSource trait이 Send + Sync을 상속받아야 하는 것이 아닌가,,?
}

#[tonic::async_trait]
impl Search for SearchService {
    type GetBooksStream = ReceiverStream<Result<Book, Status>>;

    async fn get_books(
        &self,
        request: Request<GetBooksPayload>,
    ) -> Result<Response<Self::GetBooksStream>, Status> {
        let (tx, rx) = mpsc::channel(4);

        let datasources = Arc::clone(&self.datasources);
        tokio::spawn(async move {
            for src in datasources.iter() {
                if let Ok(res) = src.search(&request.get_ref().keyword).await {
                    for book in res {
                        tx.send(Ok(book)).await.unwrap();
                    }
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    static ES_HOST: &'static str = env!("ES_HOST", "ES_HOST required");
    let es_transport = ESTransport::single_node(ES_HOST)?;
    let config = Config {
        elasticsearch: ESClient::new(es_transport),
    };
    let elasticsearch = Elasticsearch {
        es: config.elasticsearch,
    };

    let search = SearchService {
        datasources: Arc::new(vec![Box::new(elasticsearch)]),
    };
    let svc = SearchServer::new(search);

    Server::builder()
        .add_service(svc)
        .serve("127.0.0.1:56923".parse().unwrap())
        .await?;

    Ok(())
}
