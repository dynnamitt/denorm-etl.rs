/// Mostly a borrow from AWS examples
/// lots of dead_code to be removed for this use-case (later)
use aws_sdk_s3::error::ProvideErrorMetadata;
use aws_sdk_s3::operation::get_object::GetObjectOutput;
use aws_sdk_s3::operation::put_object::PutObjectOutput;
use aws_sdk_s3::primitives::{ByteStream, SdkBody};
use aws_sdk_s3::Client;

pub async fn upload_object(
    client: &Client,
    bucket_name: &str,
    contents: impl AsRef<[u8]>,
    key: String,
) -> Result<PutObjectOutput, S3Error> {
    // let body = ByteStream::from_path(Path::new(file_name)).await;
    let body = ByteStream::new(SdkBody::from(contents.as_ref()));
    client
        .put_object()
        .bucket(bucket_name)
        .key(key.to_string())
        .body(body)
        .send()
        .await
        .map_err(S3Error::from)
}

#[allow(dead_code)]
pub async fn download_object(
    client: &Client,
    bucket_name: &str,
    key: &str,
) -> Result<GetObjectOutput, S3Error> {
    client
        .get_object()
        .bucket(bucket_name)
        .key(key)
        .send()
        .await
        .map_err(S3Error::from)
}

/// Copy an object from one bucket to another.
#[allow(dead_code)]
pub async fn copy_object(
    client: &Client,
    source_bucket: &str,
    destination_bucket: &str,
    source_object: &str,
    destination_object: &str,
) -> Result<(), S3Error> {
    let source_key = format!("{source_bucket}/{source_object}");
    let response = client
        .copy_object()
        .copy_source(&source_key)
        .bucket(destination_bucket)
        .key(destination_object)
        .send()
        .await?;

    println!(
        "Copied from {source_key} to {destination_bucket}/{destination_object} with etag {}",
        response
            .copy_object_result
            .unwrap_or_else(|| aws_sdk_s3::types::CopyObjectResult::builder().build())
            .e_tag()
            .unwrap_or("missing")
    );
    Ok(())
}

#[allow(dead_code)]
pub async fn list_objects(client: &Client, bucket: &str) -> Result<(), S3Error> {
    let mut response = client
        .list_objects_v2()
        .bucket(bucket.to_owned())
        .max_keys(10) // In this example, go 10 at a time.
        .into_paginator()
        .send();

    while let Some(result) = response.next().await {
        match result {
            Ok(output) => {
                for object in output.contents() {
                    println!(" - {}", object.key().unwrap_or("Unknown"));
                }
            }
            Err(err) => {
                eprintln!("{err:?}")
            }
        }
    }

    Ok(())
}

/// Given a bucket, remove all objects in the bucket, and then ensure no objects
/// remain in the bucket.
#[allow(dead_code)]
pub async fn clear_bucket(client: Client, bucket_name: &str) -> Result<Vec<String>, S3Error> {
    let objects = client.list_objects_v2().bucket(bucket_name).send().await?;

    // delete_objects no longer needs to be mutable.
    let objects_to_delete: Vec<String> = objects
        .contents()
        .iter()
        .filter_map(|obj| obj.key())
        .map(String::from)
        .collect();

    if objects_to_delete.is_empty() {
        return Ok(vec![]);
    }

    let return_keys = objects_to_delete.clone();

    delete_objects(&client, bucket_name, objects_to_delete).await?;

    let objects = client.list_objects_v2().bucket(bucket_name).send().await?;

    eprintln!("{objects:?}");

    match objects.key_count {
        Some(0) => Ok(return_keys),
        _ => Err(S3Error::new("There were still objects left in the bucket.")),
    }
}

/// Delete the objects in a bucket.
#[allow(dead_code)]
pub async fn delete_objects(
    client: &Client,
    bucket_name: &str,
    objects_to_delete: Vec<String>,
) -> Result<(), S3Error> {
    // Push into a mut vector to use `?` early return errors while building object keys.
    let mut delete_object_ids: Vec<aws_sdk_s3::types::ObjectIdentifier> = vec![];
    for obj in objects_to_delete {
        let obj_id = aws_sdk_s3::types::ObjectIdentifier::builder()
            .key(obj)
            .build()
            .map_err(|err| {
                S3Error::new(format!("Failed to build key for delete_object: {err:?}"))
            })?;
        delete_object_ids.push(obj_id);
    }

    client
        .delete_objects()
        .bucket(bucket_name)
        .delete(
            aws_sdk_s3::types::Delete::builder()
                .set_objects(Some(delete_object_ids))
                .build()
                .map_err(|err| {
                    S3Error::new(format!("Failed to build delete_object input {err:?}"))
                })?,
        )
        .send()
        .await?;
    Ok(())
}

/// client-specific error details. This serves as a consistent backup to handling
/// specific service errors, depending on what is needed by the scenario.
/// (It is used throughout the code examples for the AWS SDK for Rust.)
///   so we copied it :)
#[derive(Debug)]
pub struct S3Error(String);

impl S3Error {
    pub fn new(value: impl Into<String>) -> Self {
        S3Error(value.into())
    }

    #[allow(dead_code)]
    pub fn add_message(self, message: impl Into<String>) -> Self {
        S3Error(format!("{}: {}", message.into(), self.0))
    }
}
// TODO: Make this generic code more generic
impl<T: ProvideErrorMetadata> From<T> for S3Error {
    fn from(value: T) -> Self {
        S3Error(format!(
            "{}: {}",
            value
                .code()
                .map(String::from)
                .unwrap_or("unknown code".into()),
            value
                .message()
                .map(String::from)
                .unwrap_or("missing reason".into()),
        ))
    }
}

impl std::error::Error for S3Error {}

impl std::fmt::Display for S3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
