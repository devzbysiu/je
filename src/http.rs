use crate::cfg::Instance;
use base64::encode;
use reqwest::blocking::multipart;
use reqwest::blocking::{Client as HttpClient, Response};
use reqwest::Result as RqRes;
use std::path::Path;

pub(crate) trait Client {
    fn get<S: Into<String>>(&self, path: S) -> RqRes<Response>;
    fn post<S: Into<String>>(&self, path: S) -> RqRes<Response>;
    fn post_file<S: Into<String>, A: AsRef<Path>>(&self, path: S, filepath: A) -> RqRes<Response>;
}

pub(crate) struct AemClient<'a> {
    instance: &'a Instance,
}

impl<'a> AemClient<'a> {
    pub(crate) fn new(instance: &'a Instance) -> Self {
        Self { instance }
    }
}

fn encoded_creds(ins: &Instance) -> String {
    encode(format!("{}:{}", ins.user(), ins.pass()))
}

impl Client for AemClient<'_> {
    fn get<S: Into<String>>(&self, path: S) -> RqRes<Response> {
        let ins = self.instance;
        let path = format!("{}{}", ins.addr(), path.into());
        let client = HttpClient::new();
        client
            .get(&path)
            .header("Authorization", format!("Basic {}", encoded_creds(ins)))
            .send()
    }

    fn post<S: Into<String>>(&self, path: S) -> RqRes<Response> {
        let ins = self.instance;
        let path = format!("{}{}", ins.addr(), path.into());
        let client = HttpClient::new();
        client
            .post(&path)
            .header("Authorization", format!("Basic {}", encoded_creds(ins)))
            .send()
    }

    fn post_file<S: Into<String>, A: AsRef<Path>>(&self, path: S, filepath: A) -> RqRes<Response> {
        let ins = self.instance;
        let path = format!("{}{}", ins.addr(), path.into());
        let client = HttpClient::new();
        let form = multipart::Form::new()
            .file("package", filepath)
            .expect("failed to create multipart form");
        client
            .post(&path)
            .header("Authorization", format!("Basic {}", encoded_creds(ins)))
            .multipart(form)
            .send()
    }
}
