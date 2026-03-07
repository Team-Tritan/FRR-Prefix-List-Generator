use thiserror::Error;

#[derive(Error, Debug)]
pub enum PrefixGenError {
    #[error("vtysh command failed: {0}")]
    VtyshError(String),
    
    #[error("vtysh not found in PATH")]
    VtyshNotFound,
    
    #[error("bgpq4 command failed for {as_set}: {reason}")]
    Bgpq4Error { as_set: String, reason: String },
    
    #[error("bgpq4 not found in PATH")]
    Bgpq4NotFound,
    
    #[error("PeeringDB API error: {status} - {message}")]
    PeeringDbError { status: u16, message: String },
    
    #[error("PeeringDB rate limit exceeded, retry after {retry_after}s")]
    PeeringDbRateLimit { retry_after: u64 },
    
    #[error("Invalid ASN: {0}")]
    InvalidAsn(String),
    
    #[error("Invalid IP address: {0}")]
    InvalidIp(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Timeout after {0}s")]
    Timeout(u64),
}

pub type Result<T> = std::result::Result<T, PrefixGenError>;
