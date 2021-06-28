use super::common::DataSource;
use crate::{Book, Library};
use async_trait::async_trait;
use elasticsearch::{Elasticsearch as ElasticsearchClient, SearchParts};
use serde::Deserialize;
use serde_json::json;

const SOURCE_ID: &'static str = "ES";

pub struct Elasticsearch {
    pub es: ElasticsearchClient,
}

#[derive(Debug, Deserialize)]
struct ESResult {
    hits: HitResult,
}

#[derive(Debug, Deserialize)]
struct HitResult {
    total: HitTotal,
    max_score: f32,
    hits: Vec<Hit>,
}

#[derive(Debug, Deserialize)]
struct HitTotal {
    value: i32,
    relation: String,
}

#[derive(Debug, Deserialize)]
struct Hit {
    _index: String,
    _type: String,
    _id: String,
    _score: f32,
    _source: Source,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Source {
    addition_symbol: String,
    authors: String,
    book_count: i32,
    isbn: String,
    kdc: String,
    lib_code: String,
    loan_count: i32,
    publication_year: String,
    publisher: String,
    reg_date: String,
    set_isbn: String,
    title: String,
    vol: String,
}

#[async_trait]
impl DataSource for Elasticsearch {
    async fn search(
        &self,
        keyword: &str,
    ) -> Result<Vec<Book>, Box<dyn std::error::Error + Send + Sync>> {
        let res = self
            .es
            .search(SearchParts::Index(&["book"]))
            .body(json!({
                "query": {
                    "bool": {
                        "must": {
                            "multi_match": {
                                "fields": [
                                    "title.nori^1.1",
                                    "authors.nori",
                                    "publisher.nori^0.8"
                                ],
                                "query": keyword,
                                "analyzer": "nori-default",
                            },
                        },
                        "should": [
                            // TODO:
                            {"term": {"libCode": {"value": "111101", "boost": 2.0}}},
                            {"term": {"libCode": {"value": "111470", "boost": 2.0}}}
                        ],
                    },
                },
            }))
            .send()
            .await?
            .json::<ESResult>()
            .await?;

        Ok(res
            .hits
            .hits
            .into_iter()
            .map(|h| {
                let s = h._source;
                Book {
                    id: format!("{}::{}", SOURCE_ID, h._id),
                    title: s.title,
                    authors: s.authors,
                    publisher: s.publisher,
                    library: Some(Library {
                        id: format!("{}::{}", SOURCE_ID, s.lib_code),
                        name: "".to_owned(),
                        address: "".to_owned(),
                        closed: "".to_owned(),
                        homepage: "".to_owned(),
                        operating_time: "".to_owned(),
                        tel: "".to_owned(),
                        location: None,
                    }),
                    link: "".to_owned(),
                    addition_symbol: s.addition_symbol,
                    book_count: s.book_count,
                    loan_count: s.loan_count,
                    isbn: s.isbn,
                    set_isbn: s.set_isbn,
                    kdc: s.kdc,
                    publication_year: s.publication_year,
                    reg_date: s.reg_date,
                    vol: s.vol,
                }
            })
            .collect())
    }
}
