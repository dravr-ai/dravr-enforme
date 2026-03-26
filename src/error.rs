// ABOUTME: Structured error types for health data sync operations
// ABOUTME: Uses thiserror for derive, no anyhow allowed
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

/// All errors that can occur during sync operations.
#[derive(Debug, thiserror::Error)]
pub enum EnformeError {
    /// A provider-specific error occurred during API communication.
    #[error("provider '{provider}' error: {message}")]
    ProviderError {
        /// The provider that caused the error.
        provider: String,
        /// Human-readable error description.
        message: String,
    },

    /// Webhook signature validation failed.
    #[error("webhook validation failed for '{provider}': {reason}")]
    WebhookValidationFailed {
        /// The provider whose webhook failed validation.
        provider: String,
        /// Reason the validation failed.
        reason: String,
    },

    /// A sync cursor was not found for the given combination.
    #[error(
        "sync cursor not found for user '{user_id}', provider '{provider}', type '{data_type}'"
    )]
    CursorNotFound {
        /// The user who owns the cursor.
        user_id: String,
        /// The provider for the cursor.
        provider: String,
        /// The data type the cursor tracks.
        data_type: String,
    },

    /// The provider rate-limited the request.
    #[error("rate limited by provider '{provider}', retry after {retry_after_secs}s")]
    RateLimited {
        /// The provider that rate-limited the request.
        provider: String,
        /// Seconds to wait before retrying.
        retry_after_secs: u64,
    },

    /// OAuth credentials have expired and need refresh.
    #[error("credentials expired for user '{user_id}', provider '{provider}'")]
    CredentialsExpired {
        /// The user whose credentials expired.
        user_id: String,
        /// The provider whose credentials expired.
        provider: String,
    },

    /// A configuration error prevents operation.
    #[error("configuration error: {message}")]
    ConfigurationError {
        /// Description of the configuration issue.
        message: String,
    },

    /// Serialization or deserialization failed.
    #[error("serialization error: {reason}")]
    SerializationError {
        /// Description of what failed to serialize.
        reason: String,
    },

    /// An I/O or network error occurred.
    #[error("network error: {message}")]
    NetworkError {
        /// Description of the network failure.
        message: String,
    },

    /// A store operation failed.
    #[error("store error: {message}")]
    StoreError {
        /// Description of the store failure.
        message: String,
    },
}

/// Convenience type alias for enforme results.
pub type EnformeResult<T> = Result<T, EnformeError>;

impl EnformeError {
    /// Create a provider error.
    pub fn provider(provider: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ProviderError {
            provider: provider.into(),
            message: message.into(),
        }
    }

    /// Create a configuration error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::ConfigurationError {
            message: message.into(),
        }
    }

    /// Create a serialization error.
    pub fn serialization(reason: impl Into<String>) -> Self {
        Self::SerializationError {
            reason: reason.into(),
        }
    }

    /// Create a store error.
    pub fn store(message: impl Into<String>) -> Self {
        Self::StoreError {
            message: message.into(),
        }
    }
}
