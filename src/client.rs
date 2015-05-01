use std::collections::HashMap;
use std::io::Read;

use hyper::status::StatusCode;
use rustc_serialize::json;
use url::{ParseError, Url};
use url::form_urlencoded::serialize_owned;

use error::Error;
use http;
use response::Response;

#[derive(Debug)]
pub struct Client {
    root_url: String,
}

impl Client {
    pub fn new(root_url: &str) -> Result<Client, ParseError> {
        let url = try!(Url::parse(root_url));
        let client = Client {
            root_url: format!("{}", url.serialize()),
        };

        Ok(client)
    }

    pub fn default() -> Client {
        Client {
            root_url: "http://127.0.0.1:2379/".to_string(),
        }
    }

    pub fn create(&self, key: &str, value: &str, ttl: Option<u64>) -> Result<Response, Error> {
        let url = self.build_url(key);
        let mut options = vec![];

        options.push(("value".to_string(), value.to_string()));
        options.push(("prevExist".to_string(), "false".to_string()));

        if ttl.is_some() {
            options.push(("ttl".to_string(), format!("{}", ttl.unwrap())));
        }

        let body = serialize_owned(&options);

        match http::put(url, body) {
            Ok(mut response) => {
                let mut response_body = String::new();

                response.read_to_string(&mut response_body).unwrap();

                match response.status {
                    StatusCode::Created => Ok(json::decode(&response_body).unwrap()),
                    _ => Err(Error::Etcd(json::decode(&response_body).unwrap())),
                }
            },
            Err(error) => Err(Error::Http(error)),
        }
    }

    pub fn delete(&self, key: &str, recursive: bool) -> Result<Response, Error> {
        let url = self.build_url(key);
        let mut options = vec![];

        options.push(("recursive".to_string(), format!("{}", recursive)));

        let body = serialize_owned(&options);

        match http::delete(url, body) {
            Ok(mut response) => {
                let mut response_body = String::new();

                response.read_to_string(&mut response_body).unwrap();

                match response.status {
                    StatusCode::Ok => Ok(json::decode(&response_body).unwrap()),
                    _ => Err(Error::Etcd(json::decode(&response_body).unwrap())),
                }
            },
            Err(error) => Err(Error::Http(error)),
        }
    }

    pub fn get(&self, key: &str, sort: bool, recursive: bool) -> Result<Response, Error> {
        let base_url = self.build_url(key);
        let sort_string = format!("{}", sort);
        let recursive_string = format!("{}", recursive);

        let mut query_pairs = HashMap::new();

        query_pairs.insert("sorted", &sort_string[..]);
        query_pairs.insert("recursive", &recursive_string[..]);

        let mut url = Url::parse(&base_url[..]).unwrap();
        url.set_query_from_pairs(query_pairs.iter().map(|(k, v)| (*k, *v)));

        let mut response = try!(http::get(format!("{}", url)));

        let mut response_body = String::new();

        response.read_to_string(&mut response_body).unwrap();

        match response.status {
            StatusCode::Ok => Ok(json::decode(&response_body).unwrap()),
            _ => Err(Error::Etcd(json::decode(&response_body).unwrap())),
        }
    }

    // private

    fn build_url(&self, path: &str) -> String {
        format!("{}v2/keys{}", self.root_url, path)
    }

}