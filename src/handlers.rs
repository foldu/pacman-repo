use std::{collections::HashMap, path::Path, sync::Arc};

use futures::{
    compat::{Future01CompatExt, Stream01CompatExt},
    future::{FutureExt, TryFutureExt},
    stream::StreamExt,
};
use futures01::{future, Future, Stream};
use snafu::{ResultExt, Snafu};
use tokio::{
    fs::File,
    io::AsyncWriteExt,
    prelude::{AsyncRead, AsyncWrite},
};
use warp::http::StatusCode;

use crate::{Config, REPO_DIR};

#[derive(Snafu, Debug)]
enum Error {
    InvalidPass,
    Warp { source: warp::Error },
    Io { source: std::io::Error },
}

impl From<Error> for warp::Rejection {
    fn from(e: Error) -> Self {
        warp::reject::custom(e)
    }
}

pub fn handle_error(err: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(e) = err.find_cause::<Error>() {
        match e {
            Error::InvalidPass | Error::Io { .. } | Error::Warp { .. } => {
                Ok(StatusCode::BAD_REQUEST)
            }
        }
    } else {
        Err(err)
    }
}

pub fn upload(
    config: Arc<Config>,
    multipart: warp::multipart::FormData,
) -> impl Future<Item = impl warp::Reply, Error = warp::Rejection> {
    upload03(config, multipart).boxed().compat()
}

async fn upload03(
    config: Arc<Config>,
    multipart: warp::multipart::FormData,
) -> Result<StatusCode, warp::Rejection> {
    let parts = multipart.take(2).collect().compat().await.context(Warp)?;
    let mut parts = parts
        .into_iter()
        .map(|part| (part.name().to_string(), part))
        .collect::<HashMap<_, _>>();

    match (parts.remove("pass"), parts.remove("file")) {
        (Some(pass), Some(mut file)) => {
            let pass: String = pass
                .fold(
                    Vec::new(),
                    |mut acc, mut chunk| -> future::FutureResult<_, warp::Error> {
                        acc.append(&mut chunk);
                        future::ok(acc)
                    },
                )
                .compat()
                .await
                .context(Warp)
                .and_then(|pass| {
                    String::from_utf8(pass)
                        .map_err(|_| Error::InvalidPass)
                        .into()
                })?;

            if &pass == &config.password {
                let filename = file
                    .filename()
                    .ok_or_else(|| warp::Rejection::from(Error::InvalidPass))?
                    .to_string();
                let mut fh = tokio::fs::File::create(filename)
                    .await
                    .context(Io)
                    .map_err(warp::Rejection::from)?;

                let mut file = file.compat();
                while let Some(chunk) = file.next().await {
                    let chunk = chunk.context(Warp)?;
                    fh.write_all(&chunk).await.context(Io)?;
                }
                Ok(StatusCode::CREATED)
            } else {
                Ok(StatusCode::CREATED)
            }
        }
        _ => Ok(StatusCode::BAD_REQUEST),
    }
}
