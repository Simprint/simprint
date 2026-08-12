use axum::{
    extract::{self, FromRequest, Request, rejection::JsonRejection},
    http::StatusCode,
};
use serde::{Serialize, de::DeserializeOwned};

use crate::utils::Response;

/// 自定义Extractor
///
/// 不使用默认的JSON extractor, 通过该extract提取可以在提取成功或失败时完成额外的操作。
pub struct Json<T>(pub T);

impl<S, T> FromRequest<S> for Json<T>
where
    T: DeserializeOwned + Serialize,
    S: Send + Sync,
{
    type Rejection = Response<T>;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match extract::Json::<T>::from_request(req, state).await {
            Ok(body) => Ok(Self(body.0)),
            Err(rejection) => {
                let (status, message) = match rejection {
                    JsonRejection::JsonDataError(e) => {
                        eprintln!("{:?}", e);
                        (StatusCode::BAD_REQUEST, "请求参数错误")
                    }
                    JsonRejection::JsonSyntaxError(_) => {
                        (StatusCode::BAD_REQUEST, "请求参数语法错误")
                    }
                    JsonRejection::MissingJsonContentType(_) => {
                        (StatusCode::BAD_REQUEST, "缺少请求参数")
                    }
                    _ => (StatusCode::OK, ""),
                };

                Err(Response::fail_with_statu_code(Some(message), status))
            }
        }
    }
}
