mod logger;
pub use logger::*;

mod error;
pub use error::*;

#[cfg(test)]
mod test {
    const SLF4J_BINDING: &str = "https://repo1.maven.org/maven2/org/slf4j/slf4j-log4j12/1.7.9/slf4j-log4j12-1.7.9.jar";
    const SLF4J_API: &str = "https://repo1.maven.org/maven2/org/slf4j/slf4j-api/1.7.9/slf4j-api-1.7.9.jar";
    const LOG4J: &str = "https://repo1.maven.org/maven2/log4j/log4j/1.2.9/log4j-1.2.9.jar";

    use jni::{InitArgsBuilder, JNIVersion, JavaVM};
    use lazy_static::lazy_static;
    use std::path::PathBuf;
    use std::sync::Mutex;

    lazy_static! {
        pub static ref JVM: Mutex<JavaVM> = {
            let binding = download_jar(SLF4J_BINDING);
            let api = download_jar(SLF4J_API);
            let log4j = download_jar(LOG4J);

            let jvm_args = InitArgsBuilder::new()
                .version(JNIVersion::V8)
                .option("-Xcheck:jni")
                .option(&format!("-Djava.class.path={}", binding.to_str().expect("Failed to convert slf4j.jar path to &str")))
                .option(&format!("-Djava.class.path={}", api.to_str().expect("Failed to convert slf4j.jar path to &str")))
                .option(&format!("-Djava.class.path={}", log4j.to_str().expect("Failed to convert slf4j.jar path to &str")))
                .build()
                .unwrap();

            let java_vm = JavaVM::new(jvm_args).expect("Failed to create JavaVM");
            Mutex::new(java_vm)
        };
    }

    fn download_jar(url: &str) -> PathBuf {
        let tmpdir = tempfile::tempdir().expect("Failed to create temporary directory");
        let response = reqwest::blocking::get(url).expect("Failed to donwload SLF4j");

        let pathbuf = PathBuf::from(tmpdir.into_path()).join("slf4j.jar");
        let mut dest = std::fs::File::create(&pathbuf).expect("Failed to create slf4j.jar");
        let content = response
            .bytes()
            .expect("Failed to read response bytes");
        std::io::copy(&mut &*content, &mut dest).expect("Failed to copy response bytes into slf4j.jar");
        pathbuf
    }
}
