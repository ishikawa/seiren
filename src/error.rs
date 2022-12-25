use crate::mir::NodeId;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BackendError {
    #[error("I/O failure")]
    IoFailure(#[from] io::Error),
    #[error("the node for id `{0}` is not laid out")]
    InvalidLayout(NodeId),
}
