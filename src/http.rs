use crate::cfg::Instance;
use anyhow::Result;
use base64::encode;
use bytes::Bytes;
use log::warn;
use reqwest::blocking::multipart;
use reqwest::blocking::{Client as HttpClient, Response as Resp};
use std::path::Path;

// the `Option<Resp>` here is not-so-elegant solution for mocking
// the response int the mock client implementations
#[derive(Debug)]
pub(crate) struct Response(pub(crate) Option<Resp>);

impl Response {
    pub(crate) fn bytes(self) -> Result<Bytes> {
        Ok(if let Some(resp) = self.0 {
            resp.bytes()?
        } else {
            warn!("no response field available");
            Bytes::default()
        })
    }
}

pub(crate) trait Client {
    fn get<S: Into<String>>(&self, path: S) -> Result<Response>;
    fn post<S: Into<String>>(&self, path: S) -> Result<Response>;
    fn post_file<S: Into<String>, A: AsRef<Path>>(&self, path: S, filepath: A) -> Result<Response>;
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
    fn get<S: Into<String>>(&self, path: S) -> Result<Response> {
        let ins = self.instance;
        let path = format!("{}{}", ins.addr(), path.into());
        let client = HttpClient::new();
        Ok(Response(Some(
            client
                .get(&path)
                .header("Authorization", format!("Basic {}", encoded_creds(ins)))
                .send()?,
        )))
    }

    fn post<S: Into<String>>(&self, path: S) -> Result<Response> {
        let ins = self.instance;
        let path = format!("{}{}", ins.addr(), path.into());
        let client = HttpClient::new();
        Ok(Response(Some(
            client
                .post(&path)
                .header("Authorization", format!("Basic {}", encoded_creds(ins)))
                .send()?,
        )))
    }

    fn post_file<S: Into<String>, A: AsRef<Path>>(&self, path: S, filepath: A) -> Result<Response> {
        let ins = self.instance;
        let path = format!("{}{}", ins.addr(), path.into());
        let client = HttpClient::new();
        let form = multipart::Form::new()
            .file("package", filepath)
            .expect("failed to create multipart form");
        Ok(Response(Some(
            client
                .post(&path)
                .header("Authorization", format!("Basic {}", encoded_creds(ins)))
                .multipart(form)
                .send()?,
        )))
    }
}
