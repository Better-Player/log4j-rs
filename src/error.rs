use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;


#[derive(Debug, Error)]
pub enum Error {
    #[error("JNI call failed")]
    Java(
        #[from]
        #[source]
        jni::errors::Error,
    ),
}
