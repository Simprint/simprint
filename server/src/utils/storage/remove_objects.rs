// use minio::s3::types::S3Api;

// /// 从对象存储中删除对象
// pub async fn remove_csgo_mvp_music_object(
//     bucket_name: &str,
//     object: &str,
// ) -> Result<(), anyhow::Error> {
//     let client = crate::minios::get_storage_client()?;

//     client.delete_object(bucket_name, object).send().await?;

//     Ok(())
// }
