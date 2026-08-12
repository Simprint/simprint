use axum::{
    extract::multipart::MultipartError,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::{Debug, Display};

const SUCCESS_CODE: i32 = 1;
const FAIL_CODE: i32 = 0;

pub type Result<T> = axum::response::Result<Response<T>, Response<String>>;

/// 统一响应类型
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Response<T> {
    pub status_code: u16,
    pub code: i32,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T> Response<T> {
    /// 操作成功对应的响应类型
    pub fn success(message: Option<&str>, data: Option<T>) -> Self {
        Response {
            code: SUCCESS_CODE,
            message: message.map(|v| v.to_string()),
            data,
            status_code: 200,
        }
    }

    /// 操作失败对应的响应类型
    pub fn fail(message: Option<&str>) -> Self {
        Response {
            code: FAIL_CODE,
            message: message.map(|v| v.to_string()),
            data: None,
            status_code: 200,
        }
    }

    /// 操作失败对应的响应类型
    pub fn fail_with_statu_code(message: Option<&str>, statu_code: StatusCode) -> Self {
        Response {
            code: FAIL_CODE,
            message: message.map(|v| v.to_string()),
            data: None,
            status_code: statu_code.as_u16(),
        }
    }

    /// with status code
    pub fn with_status_code(self, status_code: StatusCode) -> Self {
        let mut response = self;
        response.status_code = status_code.as_u16();
        response
    }
}

// 允许直接打印和to_string
impl<T> Display for Response<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

// 当实现该类型后可以直接将Response<T>作为axum路由处理函数的返回值，会自动调用该trait的into_response方法最终返回json.
impl<T> IntoResponse for Response<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        let mut content_json = json!({ "code": self.code });

        // 如果 message 存在，添加到 content_json
        if let Some(message) = &self.message {
            content_json["message"] = json!(message);
        }

        // 如果 data 存在，添加到 content_json
        if let Some(data) = &self.data {
            content_json["data"] = json!(data);
        }

        match StatusCode::from_u16(self.status_code) {
            Ok(status) => (status, Json(content_json)),
            Err(_) => (StatusCode::BAD_REQUEST, Json(content_json)),
        }
        .into_response()
    }
}

/// 实现From<MultipartError>
impl From<MultipartError> for Response<()> {
    fn from(value: MultipartError) -> Self {
        Response::fail_with_statu_code(
            Some(format!("请求参数错误: {:?}", value.body_text()).as_ref()),
            value.status(),
        )
    }
}
