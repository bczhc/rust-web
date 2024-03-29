#![feature(try_blocks)]

use axum::response::{IntoResponse, Response};
use axum::Json;
use std::path::Path;
use std::sync::Mutex;

use figment::providers::{Format, Toml};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub mod blake3;
pub mod cli;
pub mod routes;
pub mod security;

type LazyOption<T> = Lazy<Mutex<Option<T>>>;

#[macro_export]
macro_rules! lazy_option_initializer {
    () => {
        Lazy::new(|| Mutex::new(None))
    };
}

pub static CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| Mutex::new(Default::default()));
pub static ROUTES: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[macro_export]
macro_rules! mutex_lock {
    ($e:expr) => {
        $e.lock().unwrap()
    };
}

#[macro_export]
macro_rules! print_flush {
    () => {
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    };
    ($($arg:tt)*) => {
        print!($($arg)*);
        print_flush!();
    };
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    pub ccit_info_file: Option<String>,
    pub server_network_log_file: Option<String>,
    pub some_tools: Option<SomeToolsAppConfig>,
    pub diary: Option<DiaryConfig>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct SomeToolsAppConfig {
    pub crash_report_dir: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct DiaryConfig {
    pub database_file: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct ServerConfig {
    pub addr: Option<String>,
    pub port: u16,
}

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
}

pub fn read_config(config_path: impl AsRef<Path>) -> anyhow::Result<Config> {
    Ok(figment::Figment::new()
        .merge(Toml::file(config_path))
        .extract()?)
}

#[derive(Serialize)]
pub struct ResponseJson<T>
where
    T: Serialize,
{
    status: u32,
    message: Option<String>,
    data: Option<T>,
}

impl<T> ResponseJson<T>
where
    T: Serialize,
{
    pub fn error<S>(status: u32, message: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            status,
            data: None,
            message: Some(message.into()),
        }
    }

    pub fn ok(data: T) -> Self {
        Self {
            status: 0,
            message: Some("OK".into()),
            data: Some(data),
        }
    }
}

impl<T> IntoResponse for ResponseJson<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

impl<T: Serialize> Into<Response> for ResponseJson<T> {
    fn into(self) -> Response {
        self.into_response()
    }
}
