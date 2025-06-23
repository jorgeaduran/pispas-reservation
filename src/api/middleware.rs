//! # Utilidades de logging para errores
//!
//! Este módulo provee herramientas simples para demostrar thiserror en acción

use std::error::Error as StdError;

/// Registra la cadena completa de errores usando la funcionalidad de thiserror
///
/// # Parámetros
/// - `error`: Error a analizar y registrar
/// - `context`: Contexto opcional para añadir información
///
/// # Ejemplo
/// ```rust
/// use crate::api::middleware::log_error_chain;
///
/// if let Err(e) = some_operation().await {
///     log_error_chain(&e, Some("during login"));
/// }
/// ```
pub fn log_error_chain<E>(error: &E, context: Option<&str>)
where
    E: StdError + 'static,
{
    let mut error_chain = Vec::new();
    let mut current_error: Option<&dyn StdError> = Some(error);

    while let Some(err) = current_error {
        error_chain.push(err.to_string());
        current_error = err.source();
    }

    if let Some(ctx) = context {
        tracing::error!(
            context = %ctx,
            error_chain = ?error_chain,
            error_types = ?error_chain.iter().enumerate().collect::<Vec<_>>(),
            "Error with full chain (with context)"
        );
    } else {
        tracing::error!(
            error_chain = ?error_chain,
            error_types = ?error_chain.iter().enumerate().collect::<Vec<_>>(),
            "Error with full chain"
        );
    }
}

/// Extension trait para Results que añade logging automático de error chains
///
/// # Ejemplo de uso
/// ```rust
/// use crate::api::middleware::ErrorLogExt;
///
/// some_operation()
///     .await
///     .log_error_context("during database operation")?;
/// ```
pub trait ErrorLogExt<T, E> {
    /// Loggea la cadena de errores si hay un error, sin contexto adicional
    fn log_error_chain(self) -> Result<T, E>;

    /// Loggea la cadena de errores con contexto adicional
    fn log_error_context(self, context: &str) -> Result<T, E>;

    /// Loggea la cadena de errores con un nivel específico
    fn log_error_level(self, level: tracing::Level) -> Result<T, E>;
}

impl<T, E> ErrorLogExt<T, E> for Result<T, E>
where
    E: StdError + 'static,
{
    fn log_error_chain(self) -> Result<T, E> {
        if let Err(ref error) = self {
            log_error_chain(error, None);
        }
        self
    }

    fn log_error_context(self, context: &str) -> Result<T, E> {
        if let Err(ref error) = self {
            log_error_chain(error, Some(context));
        }
        self
    }

    fn log_error_level(self, level: tracing::Level) -> Result<T, E> {
        if let Err(ref error) = self {
            match level {
                tracing::Level::ERROR => log_error_chain(error, None),
                tracing::Level::WARN => {
                    let mut error_chain = Vec::new();
                    let mut current_error: Option<&dyn StdError> = Some(error);

                    while let Some(err) = current_error {
                        error_chain.push(err.to_string());
                        current_error = err.source();
                    }

                    tracing::warn!(
                        error_chain = ?error_chain,
                        "Warning with error chain"
                    );
                }
                _ => {
                    tracing::info!("Error occurred: {}", error);
                }
            }
        }
        self
    }
}

/// Macro helper para logging de errores contextualizados
///
/// # Ejemplo
/// ```rust
/// log_error_context!(result, "during user authentication");
/// ```
#[macro_export]
macro_rules! log_error_context {
    ($result:expr, $context:expr) => {
        match &$result {
            Ok(_) => {},
            Err(e) => {
                $crate::api::middleware::log_error_chain(e, Some($context));
            }
        }
    };
}