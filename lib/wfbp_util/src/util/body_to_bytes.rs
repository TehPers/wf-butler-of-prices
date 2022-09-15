use crate::errors::ReadBodyError;
use axum::body::{Bytes, HttpBody};

/// Reads an entire [`HttpBody`] into a single [`Bytes`] instance.
pub async fn body_to_bytes<B>(mut body: B) -> Result<Bytes, ReadBodyError>
where
    B: HttpBody<Data = Bytes> + Unpin,
    B::Error: Into<anyhow::Error>,
{
    // If only one chunk, return it
    let first_chunk = match body
        .data()
        .await
        .transpose()
        .map_err(ReadBodyError::from_error)?
    {
        Some(chunk) => chunk,
        None => return Ok(Bytes::new()),
    };
    let second_chunk = match body
        .data()
        .await
        .transpose()
        .map_err(ReadBodyError::from_error)?
    {
        Some(chunk) => chunk,
        None => return Ok(first_chunk),
    };

    // Collect all the chunks into a buffer
    let capacity = body
        .size_hint()
        .lower()
        .saturating_add(first_chunk.len().try_into().unwrap_or(0))
        .saturating_add(second_chunk.len().try_into().unwrap_or(0));
    let mut data = Vec::with_capacity(capacity.min(2048) as usize);
    data.extend_from_slice(&first_chunk);
    data.extend_from_slice(&second_chunk);
    while let Some(chunk) = body.data().await {
        let chunk = chunk.map_err(ReadBodyError::from_error)?;
        data.extend_from_slice(&chunk);
    }

    Ok(data.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use futures::StreamExt;

    #[tokio::test]
    async fn single_chunk() {
        let body = Body::from(Bytes::from(b"hello world".as_slice()));
        let bytes = body_to_bytes(body).await.unwrap();
        assert_eq!(bytes, Bytes::from_static(b"hello world"));
    }

    #[tokio::test]
    async fn multiple_chunks() {
        let chunks = futures::stream::iter([
            b"hello".as_slice(),
            b" ".as_slice(),
            b"world".as_slice(),
        ])
        .map(|s| Ok(Bytes::from(s)));
        let body = Body::from(Box::new(chunks) as Box<_>);
        let bytes = body_to_bytes(body).await.unwrap();
        assert_eq!(bytes, Bytes::from_static(b"hello world"));
    }

    #[tokio::test]
    async fn empty() {
        let body = Body::empty();
        let bytes = body_to_bytes(body).await.unwrap();
        assert_eq!(bytes, Bytes::new());
    }

    #[tokio::test]
    async fn error() {
        let chunks = futures::stream::iter([
            Ok(Bytes::from_static(b"hello")),
            Err("error".into()),
        ]);
        let body = Body::from(Box::new(chunks) as Box<_>);
        let bytes = body_to_bytes(body).await;
        assert!(bytes.is_err());
    }
}
