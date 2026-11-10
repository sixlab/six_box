use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

pub const MAX_FRAME_BYTES: usize = 65536;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeError {
    pub code: String,
    pub message: String,
}
impl RuntimeError {
    pub fn new(code: &str, message: impl ToString) -> Self {
        Self {
            code: code.into(),
            message: message.to_string(),
        }
    }
}
pub async fn read_frame<R: AsyncBufRead + Unpin>(reader: &mut R) -> Result<Value, RuntimeError> {
    let mut frame = Vec::new();
    loop {
        let bytes = reader
            .fill_buf()
            .await
            .map_err(|e| RuntimeError::new("READ_FAILED", e))?;
        if bytes.is_empty() {
            return Err(RuntimeError::new(
                if frame.is_empty() {
                    "DISCONNECTED"
                } else {
                    "TRUNCATED_FRAME"
                },
                "管理服务通道已关闭",
            ));
        }
        let newline = bytes.iter().position(|b| *b == b'\n');
        let length = newline.unwrap_or(bytes.len());
        if frame.len() + length > MAX_FRAME_BYTES {
            return Err(RuntimeError::new(
                "FRAME_TOO_LARGE",
                "管理服务消息超过 64 KiB",
            ));
        }
        frame.extend_from_slice(&bytes[..length]);
        reader.consume(length + usize::from(newline.is_some()));
        if newline.is_some() {
            return serde_json::from_slice(&frame)
                .map_err(|_| RuntimeError::new("PROTOCOL_ERROR", "管理服务返回无效 JSON"));
        }
    }
}
pub async fn write_frame<W: AsyncWrite + Unpin>(
    writer: &mut W,
    message: &Value,
) -> Result<(), RuntimeError> {
    let mut bytes =
        serde_json::to_vec(message).map_err(|e| RuntimeError::new("PROTOCOL_ERROR", e))?;
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(RuntimeError::new("FRAME_TOO_LARGE", "请求超过 64 KiB"));
    }
    bytes.push(b'\n');
    writer
        .write_all(&bytes)
        .await
        .map_err(|e| RuntimeError::new("WRITE_FAILED", e))?;
    writer
        .flush()
        .await
        .map_err(|e| RuntimeError::new("WRITE_FAILED", e))
}
pub fn response_result(frame: Value, id: u64) -> Result<Value, RuntimeError> {
    if frame["version"] != 1
        || frame["type"] != "response"
        || frame["id"].as_u64() != Some(id)
        || frame.get("result").is_some() == frame.get("error").is_some()
    {
        return Err(RuntimeError::new(
            "PROTOCOL_ERROR",
            "管理服务响应格式、版本或 ID 不匹配",
        ));
    }
    if let Some(error) = frame.get("error") {
        return Err(serde_json::from_value(error.clone())
            .unwrap_or_else(|_| RuntimeError::new("PROTOCOL_ERROR", "管理服务错误格式无效")));
    }
    Ok(frame["result"].clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncWriteExt, BufReader};
    #[tokio::test]
    async fn reads_json_and_rejects_large_truncated_or_invalid_frames() {
        assert_eq!(
            read_frame(&mut &b"{\"version\":1}\n"[..]).await.unwrap()["version"],
            1
        );
        for (bytes, code) in [
            (vec![b'x'; MAX_FRAME_BYTES + 1], "FRAME_TOO_LARGE"),
            (b"{}".to_vec(), "TRUNCATED_FRAME"),
            (b"oops\n".to_vec(), "PROTOCOL_ERROR"),
            (vec![], "DISCONNECTED"),
        ] {
            assert_eq!(read_frame(&mut &bytes[..]).await.unwrap_err().code, code);
        }
    }
    #[test]
    fn validates_ids_versions_and_remote_errors() {
        assert_eq!(
            response_result(
                serde_json::json!({"version":1,"type":"response","id":7,"result":{"alive":true}}),
                7
            )
            .unwrap()["alive"],
            true
        );
        for frame in [
            serde_json::json!({"version":1,"type":"response","id":8,"result":{}}),
            serde_json::json!({"version":2,"type":"response","id":7,"result":{}}),
            serde_json::json!({"version":1,"type":"response","id":7}),
            serde_json::json!({"version":1,"type":"response","id":7,"result":{},"error":{"code":"X","message":"x"}}),
        ] {
            assert_eq!(
                response_result(frame, 7).unwrap_err().code,
                "PROTOCOL_ERROR"
            );
        }
        assert_eq!(response_result(serde_json::json!({"version":1,"type":"response","id":7,"error":{"code":"INVALID_PARAMS","message":"invalid"}}),7).unwrap_err().code,"INVALID_PARAMS");
    }
    #[tokio::test]
    async fn write_failure_is_reported_and_fragmented_frames_work() {
        let (mut tx, rx) = tokio::io::duplex(64);
        drop(rx);
        assert_eq!(
            write_frame(&mut tx, &serde_json::json!({"id":1}))
                .await
                .unwrap_err()
                .code,
            "WRITE_FAILED"
        );
        let (mut tx, rx) = tokio::io::duplex(64);
        tokio::spawn(async move {
            tx.write_all(b"{\"v\":").await.unwrap();
            tokio::task::yield_now().await;
            tx.write_all(b"1}\n").await.unwrap();
        });
        assert_eq!(read_frame(&mut BufReader::new(rx)).await.unwrap()["v"], 1);
    }
}
