use crate::error::Result;
use jni::objects::{JMethodID, JObject, JValue};
use jni::signature::{JavaType, Primitive};
use jni::JNIEnv;
use std::sync::{Arc, Mutex};

const LOG_MANAGER_CLASS: &str = "org/apache/log4j/LogManager";
const CATEGORY_CLASS: &str = "org/apache/log4j/Category";

struct InnerLogger<'a> {
    env:            &'a JNIEnv<'a>,
    logger:         JObject<'a>,         // This is an instance of org.apache.log4j.Logger
    info_method:    JMethodID<'a>,  // Logger#info(Object)
    error_method:   JMethodID<'a>, // Logger#error(Object)
    warn_method:    JMethodID<'a>,  // Logger#warn(Object)
    debug_method:   JMethodID<'a>, // Logger#debug(Object)
}

/// The JavaLogger
pub struct JavaLogger<'a> {
    inner: Arc<Mutex<InnerLogger<'a>>>,
}

// Required because the compiler does not pick up that JavaLogger can be Send+Sync
// InnerLogger is !Send + !Sync, but we're wrapping it in an Arc<Mutex<T>>
unsafe impl<'a> Send for JavaLogger<'a> {}
unsafe impl<'a> Sync for JavaLogger<'a> {}

/// The log level to output to
pub enum LogLevel {
    /// ERROR level
    Error,
    /// WARN level
    Warn,
    /// INFO level
    Info,
    /// DEBUG level, also applicable to TRACE logging
    Debug,
}

impl<'a> JavaLogger<'a> {
    /// Create a new logger
    ///
    /// # Params
    /// - `class_name` The name of Class which should be used by log4j on the Java side
    ///
    /// # Error
    /// - If one of the underlying JNI calls fail
    pub fn new<S: AsRef<str>>(env: &'a JNIEnv<'a>, class_name: S) -> Result<Self> {
        let log_manager_class = env.find_class(LOG_MANAGER_CLASS)?;
        let logger_value = env.call_static_method(log_manager_class,             "getLogger","(Ljava/lang/String;)Lorg/apache/log4j/Logger;",&[Self::jstring(env, class_name.as_ref())?])?;
        let logger = logger_value.l()?;

        let category_class = env.find_class(CATEGORY_CLASS)?;
        let info_method = env.get_method_id(category_class, "info", "(Ljava/lang/Object;)V")?;
        let error_method = env.get_method_id(category_class, "error", "(Ljava/lang/Object;)V")?;
        let warn_method = env.get_method_id(category_class, "warn", "(Ljava/lang/Object;)V")?;
        let debug_method = env.get_method_id(category_class, "debug", "(Ljava/lang/Object;)V")?;

        Ok(Self {
            inner: Arc::new(Mutex::new(InnerLogger {
                env,
                logger,
                info_method,
                error_method,
                warn_method,
                debug_method,
            })),
        })
    }

    /// Log to log4j
    ///
    /// # Error
    /// - If one of the underlying JNI calls fail
    pub fn log<S: AsRef<str>>(&self, level: LogLevel, content: S) -> Result<()> {
        let logger = self.inner.lock().expect("Failed to lock inner logger");
        match level {
            LogLevel::Error => Self::log_error(&logger, content.as_ref())?,
            LogLevel::Warn => Self::log_warn(&logger, content.as_ref())?,
            LogLevel::Info => Self::log_info(&logger, content.as_ref())?,
            LogLevel::Debug => Self::log_debug(&logger, content.as_ref())?,
        };

        Ok(())
    }

    /// Log to the ERROR level
    ///
    /// # Error
    /// - If one of the underlying JNI calls fail
    fn log_error<'b>(logger: &'b InnerLogger<'a>, msg: &str) -> Result<()>
    where
        'a: 'b,
    {
        logger.env.call_method_unchecked(logger.logger,logger.error_method,JavaType::Primitive(Primitive::Void), &[Self::jstring(logger.env, msg)?])?;
        Ok(())
    }

    /// Log to the WARN level
    ///
    /// # Error
    /// - If one of the underlying JNI calls fail
    fn log_warn<'b>(logger: &'b InnerLogger<'a>, msg: &str) -> Result<()>
    where
        'a: 'b,
    {
        logger.env.call_method_unchecked(logger.logger,logger.warn_method,JavaType::Primitive(Primitive::Void),&[Self::jstring(logger.env, msg)?])?;
        Ok(())
    }

    /// Log to the INFO level
    ///
    /// # Error
    /// - If one of the underlying JNI calls fail
    fn log_info<'b>(logger: &'b InnerLogger<'a>, msg: &str) -> Result<()>
    where
        'a: 'b,
    {
        logger.env.call_method_unchecked(logger.logger,logger.info_method,JavaType::Primitive(Primitive::Void),&[Self::jstring(logger.env, msg)?])?;
        Ok(())
    }

    /// Log to the DEBUG level
    ///
    /// # Error
    /// - If one of the underlying JNI calls fail
    fn log_debug<'b>(logger: &'b InnerLogger<'a>, msg: &str) -> Result<()>
    where
        'a: 'b,
    {
        logger.env.call_method_unchecked(logger.logger,logger.debug_method,JavaType::Primitive(Primitive::Void),&[Self::jstring(logger.env, msg)?])?;
        Ok(())
    }

    /// Turn a string into a JValue containing a JString
    ///
    /// # Error
    /// - If one of the underlying JNI calls fail
    fn jstring(env: &'a JNIEnv<'a>, content: &str) -> Result<JValue<'a>> {
        let str = env.new_string(content)?;
        Ok(JValue::Object(str.into()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::JVM;

    fn setup_log4j(logger: &JavaLogger) -> Result<()> {
        let logger = logger.inner.lock().unwrap();
        let env = logger.env;

        let pattern_layout_class = env.find_class("org/apache/log4j/PatternLayout")?;
        let pattern_layout = env.new_object(pattern_layout_class,"(Ljava/lang/String;)V",&[JValue::Object(env.new_string("%r [%t] %p %c %x - %m%n")?.into())])?;

        let console_appender_class = env.find_class("org/apache/log4j/ConsoleAppender")?;
        let console_appender = env.new_object(console_appender_class,"(Lorg/apache/log4j/Layout;)V",&[JValue::Object(pattern_layout)])?;

        let category_class = env.find_class(CATEGORY_CLASS)?;
        let add_apender_method = env.get_method_id(category_class,"addAppender","(Lorg/apache/log4j/Appender;)V")?;
        env.call_method_unchecked(logger.logger,add_apender_method,JavaType::Primitive(Primitive::Void),&[JValue::Object(console_appender)])?;
        Ok(())
    }

    #[test]
    fn info() {
        let jvm = JVM.lock().expect("Failed to lock JVM");
        let env = jvm.attach_current_thread().expect("Failed to attach current thread to the JVM");
        let logger = JavaLogger::new(&env, "com.example.Info").expect("Failed to create JavaLogger");
        setup_log4j(&logger).expect("Failed to set up log4j");

        let inner_logger = logger.inner.lock().expect("Failed to lock inner logger");
        JavaLogger::log_info(&inner_logger, "Info log!").expect("Failed to log to INFO");
    }

    #[test]
    fn warn() {
        let jvm = JVM.lock().expect("Failed to lock JVM");
        let env = jvm.attach_current_thread().expect("Failed to attach current thread to the JVM");
        let logger = JavaLogger::new(&env, "com.example.Warn").expect("Failed to create JavaLogger");
        setup_log4j(&logger).expect("Failed to set up log4j");

        let inner_logger = logger.inner.lock().expect("Failed to lock inner logger");
        JavaLogger::log_warn(&inner_logger, "Warning log!").expect("Failed to log to WARN");
    }

    #[test]
    fn error() {
        let jvm = JVM.lock().expect("Failed to lock JVM");
        let env = jvm.attach_current_thread().expect("Failed to attach current thread to the JVM");
        let logger = JavaLogger::new(&env, "com.example.Error").expect("Failed to create JavaLogger");
        setup_log4j(&logger).expect("Failed to set up log4j");

        let inner_logger = logger.inner.lock().expect("Failed to lock inner logger");
        JavaLogger::log_error(&inner_logger, "Error log!").expect("Failed to log to ERROR");
    }

    #[test]
    fn trace_and_debug() {
        let jvm = JVM.lock().expect("Failed to lock JVM");
        let env = jvm.attach_current_thread().expect("Failed to attach current thread to the JVM");
        let logger = JavaLogger::new(&env, "com.example.Debug").expect("Failed to create JavaLogger");
        setup_log4j(&logger).expect("Failed to set up log4j");

        let inner_logger = logger.inner.lock().expect("Failed to lock inner logger");
        JavaLogger::log_debug(&inner_logger, "Trace and debug log!").expect("Failed to log to DEBUG");
    }

    #[test]
    fn log_general() {
        let jvm = JVM.lock().expect("Failed to lock JVM");
        let env = jvm.attach_current_thread().expect("Failed to attach current thread to the JVM");
        let logger = JavaLogger::new(&env, "com.example.General").expect("Failed to create JavaLogger");
        setup_log4j(&logger).expect("Failed to set up log4j");

        logger.log(LogLevel::Error, "Error!").expect("Failed to log to ERROR level");
        logger.log(LogLevel::Warn, "Warn!").expect("Failed to log to WARN level");
        logger.log(LogLevel::Info, "Info!").expect("Failed to log to INFO level");
        logger.log(LogLevel::Debug, "Debug!").expect("Failed to log to DEBUG level");
    }
}
