use crate::var::{ReplaceVar, ToVar, Var};
use hyper::{Body, Request};
use std::path::Path;
use std::sync::Arc;
use tokio::fs::{File, OpenOptions};
use tokio::io::*;
use tokio::io::{self, Result};
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct Logger {
    format: Var<String>,
    file: Option<Arc<Mutex<File>>>,
    stdout: Option<Arc<Mutex<Stdout>>>,
}

impl Logger {
    pub fn new<S: AsRef<str>>(format: S) -> Self {
        let mut f = format.as_ref().to_string();
        f += "\n";

        Self {
            format: f.to_var(),
            file: None,
            stdout: None,
        }
    }

    // Set output to file
    pub async fn file<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await
            .map(|file| Arc::new(Mutex::new(file)))?;
        self.file = Some(file);

        Ok(self)
    }

    // Set output to terminal stdout
    pub fn stdout(mut self) -> Self {
        self.stdout = Some(Arc::new(Mutex::new(io::stdout())));

        self
    }

    pub async fn write(&mut self, req: &Request<Body>) {
        let text = self
            .format
            .clone()
            .unwrap_or_else(|s, r| s.as_str().replace_var(&r, req));
        let bytes = text.as_bytes();

        // file
        if let Some(file) = &self.file {
            let arc = file.clone();
            let mut file = arc.lock().await;

            let _ = file.write(bytes).await;
        }

        // stdout
        if let Some(stdout) = &self.stdout {
            let arc = stdout.clone();
            let mut stdout = arc.lock().await;

            let _ = stdout.write(bytes).await;
        }
    }
}

#[tokio::test]
async fn test_logger() {}

#[tokio::test]
async fn test_logger_file() {
    use tokio::fs;
    let file = "./test.log";
    let data = "12345";

    let mut logger = Logger::new(&data).file(file).await.unwrap();
    let req = Request::new(Body::empty());
    logger.write(&req).await;

    let content = fs::read_to_string(file).await.unwrap();
    fs::remove_file(file).await.unwrap();

    assert_eq!(format!("{}\n", data), content);
}

#[tokio::test]
async fn test_logger_stdout() {}
