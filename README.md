# log4j-rs
![Crates.io](https://img.shields.io/crates/v/log4j?style=flat)  
This crate allows a Rust developer to easily log to Java's Log4j from Rust.

## Usage
```rs
use log4j::{JavaLogger, LogLevel};

// Create a logger
// This assumes that com.example.Example has an appender in the Java log4j configuration.
// `&env` is a reference to jni::JNIENv
let logger = JavaLogger::new(&env, "com.example.Example").expect("Failed to create JavaLogger");

// Now for the actual logging
logger.log(LogLevel::Error, "Error!").expect("Failed to log to ERROR level");
logger.log(LogLevel::Warn, "Warn!").expect("Failed to log to WARN level");
logger.log(LogLevel::Info, "Info!").expect("Failed to log to INFO level");
logger.log(LogLevel::Debug, "Debug!").expect("Failed to log to DEBUG level");
```

## License
`log4j-rs` is dual licensed under the Apache-2.0 and MIT license, at your discretion
