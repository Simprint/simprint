use crate::{
    services::now_service,
    utils::{Response, Result},
};

pub async fn now_handle() -> Result<String> {
    let now = now_service();

    Ok(Response::<String>::success(Some("获取时间成功"), Some(now)))
}
