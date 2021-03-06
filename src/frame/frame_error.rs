//! This modules contains [Cassandra's errors]
//! (https://github.com/apache/cassandra/blob/trunk/doc/native_protocol_v4.spec#L1011)
//! which server could respond to client.

use std::io;
use std::result;
use consistency::Consistency;
use types::*;
use FromCursor;
use frame::Frame;

/// CDRS specific `Result` which contains a [`Frame`] in case of `Ok` and `CDRSError` if `Err`.
///
/// [`Frame`]: ../frame/struct.Frame.html
pub type Result = result::Result<Frame, CDRSError>;

/// CDRS error which could be returned by Cassandra server as a response. As it goes
/// from the specification it contains an error code and an error message. Apart of those
/// depending of type of error it could contain an additional information about an error.
/// This additional information is represented by `additional_info` property which is `ErrorKind`.
#[derive(Debug)]
pub struct CDRSError {
    /// `i32` that points to a type of error.
    pub error_code: CInt,
    /// Error message string.
    pub message: CString,
    /// Additional information.
    pub additional_info: AdditionalErrorInfo,
}

impl FromCursor for CDRSError {
    fn from_cursor(mut cursor: &mut io::Cursor<&[u8]>) -> CDRSError {
        let error_code = CInt::from_cursor(&mut cursor);
        let message = CString::from_cursor(&mut cursor);
        let additional_info = AdditionalErrorInfo::from_cursor_with_code(&mut cursor, error_code);
        CDRSError {
            error_code: error_code,
            message: message,
            additional_info: additional_info,
        }
    }
}

/// Additional error info in accordance to
/// [Cassandra protocol v4]
/// (https://github.com/apache/cassandra/blob/trunk/doc/native_protocol_v4.spec#L1011).
#[derive(Debug)]
pub enum AdditionalErrorInfo {
    Server(SimpleError),
    Protocol(SimpleError),
    Authentication(SimpleError),
    Unavailable(UnavailableError),
    Overloaded(SimpleError),
    IsBootstrapping(SimpleError),
    Truncate(SimpleError),
    WriteTimeout(WriteTimeoutError),
    ReadTimeout(ReadTimeoutError),
    ReadFailure(ReadFailureError),
    FunctionFailure(FunctionFailureError),
    WriteFailure(WriteFailureError),
    Syntax(SimpleError),
    Unauthorized(SimpleError),
    Invalid(SimpleError),
    Config(SimpleError),
    AlreadyExists(AlreadyExistsError),
    Unprepared(UnpreparedError),
}

impl AdditionalErrorInfo {
    pub fn from_cursor_with_code(mut cursor: &mut io::Cursor<&[u8]>,
                                 error_code: CInt)
                                 -> AdditionalErrorInfo {
        match error_code {
            0x0000 => AdditionalErrorInfo::Server(SimpleError::from_cursor(&mut cursor)),
            0x000A => AdditionalErrorInfo::Protocol(SimpleError::from_cursor(&mut cursor)),
            0x0100 => AdditionalErrorInfo::Authentication(SimpleError::from_cursor(&mut cursor)),
            0x1000 => AdditionalErrorInfo::Unavailable(UnavailableError::from_cursor(&mut cursor)),
            0x1001 => AdditionalErrorInfo::Overloaded(SimpleError::from_cursor(&mut cursor)),
            0x1002 => AdditionalErrorInfo::IsBootstrapping(SimpleError::from_cursor(&mut cursor)),
            0x1003 => AdditionalErrorInfo::Truncate(SimpleError::from_cursor(&mut cursor)),
            0x1100 => {
                AdditionalErrorInfo::WriteTimeout(WriteTimeoutError::from_cursor(&mut cursor))
            }
            0x1200 => AdditionalErrorInfo::ReadTimeout(ReadTimeoutError::from_cursor(&mut cursor)),
            0x1300 => AdditionalErrorInfo::ReadFailure(ReadFailureError::from_cursor(&mut cursor)),
            0x1400 => {
                AdditionalErrorInfo::FunctionFailure(FunctionFailureError::from_cursor(&mut cursor))
            }
            0x1500 => {
                AdditionalErrorInfo::WriteFailure(WriteFailureError::from_cursor(&mut cursor))
            }
            0x2000 => AdditionalErrorInfo::Syntax(SimpleError::from_cursor(&mut cursor)),
            0x2100 => AdditionalErrorInfo::Unauthorized(SimpleError::from_cursor(&mut cursor)),
            0x2200 => AdditionalErrorInfo::Invalid(SimpleError::from_cursor(&mut cursor)),
            0x2300 => AdditionalErrorInfo::Config(SimpleError::from_cursor(&mut cursor)),
            0x2400 => {
                AdditionalErrorInfo::AlreadyExists(AlreadyExistsError::from_cursor(&mut cursor))
            }
            0x2500 => AdditionalErrorInfo::Unprepared(UnpreparedError::from_cursor(&mut cursor)),
            _ => unreachable!(),
        }
    }
}

/// Is used if error does not contain any additional info.
#[derive(Debug)]
pub struct SimpleError {}

impl FromCursor for SimpleError {
    fn from_cursor(mut _cursor: &mut io::Cursor<&[u8]>) -> SimpleError {
        SimpleError {}
    }
}

/// Additional info about
/// [unavailable exception]
/// (https://github.com/apache/cassandra/blob/trunk/doc/native_protocol_v4.spec#L1025)
#[derive(Debug)]
pub struct UnavailableError {
    /// Consistency level of query.
    pub cl: Consistency,
    /// Number of nodes that should be available to respect `cl`.
    pub required: CInt,
    /// Number of replicas that we were know to be alive.
    pub alive: CInt,
}

impl FromCursor for UnavailableError {
    fn from_cursor(mut cursor: &mut io::Cursor<&[u8]>) -> UnavailableError {
        let cl = Consistency::from_cursor(&mut cursor);
        let required = CInt::from_cursor(&mut cursor);
        let alive = CInt::from_cursor(&mut cursor);

        UnavailableError {
            cl: cl,
            required: required,
            alive: alive,
        }
    }
}

/// Timeout exception during a write request.
#[derive(Debug)]
pub struct WriteTimeoutError {
    /// Consistency level of query.
    pub cl: Consistency,
    /// `i32` representing the number of nodes having acknowledged the request.
    pub received: CInt,
    /// `i32` representing the number of replicas whose acknowledgement is required to achieve `cl`.
    pub blockfor: CInt,
    /// Describes the type of the write that timed out
    pub write_type: WriteType,
}

impl FromCursor for WriteTimeoutError {
    fn from_cursor(mut cursor: &mut io::Cursor<&[u8]>) -> WriteTimeoutError {
        let cl = Consistency::from_cursor(&mut cursor);
        let received = CInt::from_cursor(&mut cursor);
        let blockfor = CInt::from_cursor(&mut cursor);
        let write_type = WriteType::from_cursor(&mut cursor);

        return WriteTimeoutError {
            cl: cl,
            received: received,
            blockfor: blockfor,
            write_type: write_type,
        };
    }
}

/// Timeout exception during a read request.
#[derive(Debug)]
pub struct ReadTimeoutError {
    /// Consistency level of query.
    pub cl: Consistency,
    /// `i32` representing the number of nodes having acknowledged the request.
    pub received: CInt,
    /// `i32` representing the number of replicas whose acknowledgement is required to achieve `cl`.
    pub blockfor: CInt,
    data_present: u8,
}

impl ReadTimeoutError {
    /// Shows if replica has resonded to a query.
    pub fn replica_has_responded(&self) -> bool {
        self.data_present != 0
    }
}

impl FromCursor for ReadTimeoutError {
    fn from_cursor(mut cursor: &mut io::Cursor<&[u8]>) -> ReadTimeoutError {
        let cl = Consistency::from_cursor(&mut cursor);
        let received = CInt::from_cursor(&mut cursor);
        let blockfor = CInt::from_cursor(&mut cursor);
        let data_present = from_bytes(cursor_next_value(&mut cursor, 1).as_slice()) as u8;
        ReadTimeoutError {
            cl: cl,
            received: received,
            blockfor: blockfor,
            data_present: data_present,
        }
    }
}

/// A non-timeout exception during a read request.
#[derive(Debug)]
pub struct ReadFailureError {
    /// Consistency level of query.
    pub cl: Consistency,
    /// `i32` representing the number of nodes having acknowledged the request.
    pub received: CInt,
    /// `i32` representing the number of replicas whose acknowledgement is required to achieve `cl`.
    pub blockfor: CInt,
    /// Represents the number of nodes that experience a failure while executing the request.
    pub num_failures: CInt,
    data_present: u8,
}

impl ReadFailureError {
    /// Shows if replica has resonded to a query.
    pub fn replica_has_responded(&self) -> bool {
        self.data_present != 0
    }
}

impl FromCursor for ReadFailureError {
    fn from_cursor(mut cursor: &mut io::Cursor<&[u8]>) -> ReadFailureError {
        let cl = Consistency::from_cursor(&mut cursor);
        let received = CInt::from_cursor(&mut cursor);
        let blockfor = CInt::from_cursor(&mut cursor);
        let num_failures = CInt::from_cursor(&mut cursor);
        let data_present = from_bytes(cursor_next_value(&mut cursor, 1).as_slice()) as u8;
        ReadFailureError {
            cl: cl,
            received: received,
            blockfor: blockfor,
            num_failures: num_failures,
            data_present: data_present,
        }
    }
}

/// A (user defined) function failed during execution.
#[derive(Debug)]
pub struct FunctionFailureError {
    /// The keyspace of the failed function.
    pub keyspace: CString,
    /// The name of the failed function
    pub function: CString,
    /// `Vec<CString>` one string for each argument type (as CQL type) of the failed function.
    pub arg_types: CStringList,
}

impl FromCursor for FunctionFailureError {
    fn from_cursor(mut cursor: &mut io::Cursor<&[u8]>) -> FunctionFailureError {
        let keyspace = CString::from_cursor(&mut cursor);
        let function = CString::from_cursor(&mut cursor);
        let arg_types = CStringList::from_cursor(&mut cursor);
        FunctionFailureError {
            keyspace: keyspace,
            function: function,
            arg_types: arg_types,
        }
    }
}

/// A non-timeout exception during a write request.
/// [Read more...](https://github.com/apache/cassandra/blob/trunk/doc/native_protocol_v4.spec#L1106)
#[derive(Debug)]
pub struct WriteFailureError {
    /// Consistency of the query having triggered the exception.
    pub cl: Consistency,
    /// Represents the number of nodes having answered the request.
    pub received: CInt,
    /// Represents the number of replicas whose acknowledgement is required to achieve `cl`.
    pub blockfor: CInt,
    /// Represents the number of nodes that experience a failure while executing the request.
    pub num_failures: CInt,
    /// describes the type of the write that failed.
    pub write_type: WriteType,
}

impl FromCursor for WriteFailureError {
    fn from_cursor(mut cursor: &mut io::Cursor<&[u8]>) -> WriteFailureError {
        let cl = Consistency::from_cursor(&mut cursor);
        let received = CInt::from_cursor(&mut cursor);
        let blockfor = CInt::from_cursor(&mut cursor);
        let num_failures = CInt::from_cursor(&mut cursor);
        let write_type = WriteType::from_cursor(&mut cursor);
        WriteFailureError {
            cl: cl,
            received: received,
            blockfor: blockfor,
            num_failures: num_failures,
            write_type: write_type,
        }
    }
}

/// Describes the type of the write that failed.
/// [Read more...](https://github.com/apache/cassandra/blob/trunk/doc/native_protocol_v4.spec#L1118)
#[derive(Debug)]
pub enum WriteType {
    /// The write was a non-batched non-counter write
    Simple,
    /// The write was a (logged) batch write.
    /// If this type is received, it means the batch log
    /// has been successfully written
    Batch,
    /// The write was an unlogged batch. No batch log write has been attempted.
    UnloggedBatch,
    /// The write was a counter write (batched or not)
    Counter,
    /// The failure occured during the write to the batch log when a (logged) batch
    /// write was requested.
    BatchLog,
}

impl FromCursor for WriteType {
    fn from_cursor(mut cursor: &mut io::Cursor<&[u8]>) -> WriteType {
        match CString::from_cursor(&mut cursor).as_str() {
            "SIMPLE" => WriteType::Simple,
            "BATCH" => WriteType::Batch,
            "UNLOGGED_BATCH" => WriteType::UnloggedBatch,
            "COUNTER" => WriteType::Counter,
            "BATCH_LOG" => WriteType::BatchLog,
            _ => unreachable!(),
        }
    }
}

/// The query attempted to create a keyspace or a table that was already existing.
/// [Read more...](https://github.com/apache/cassandra/blob/trunk/doc/native_protocol_v4.spec#L1140)
#[derive(Debug)]
pub struct AlreadyExistsError {
    /// Represents either the keyspace that already exists,
    /// or the keyspace in which the table that already exists is.
    pub ks: CString,
    /// Represents the name of the table that already exists.
    pub table: CString,
}

impl FromCursor for AlreadyExistsError {
    fn from_cursor(mut cursor: &mut io::Cursor<&[u8]>) -> AlreadyExistsError {
        let ks = CString::from_cursor(&mut cursor);
        let table = CString::from_cursor(&mut cursor);

        AlreadyExistsError {
            ks: ks,
            table: table,
        }
    }
}

/// Can be thrown while a prepared statement tries to be
/// executed if the provided prepared statement ID is not known by
/// this host. [Read more...]
/// (https://github.com/apache/cassandra/blob/trunk/doc/native_protocol_v4.spec#L1150)
#[derive(Debug)]
pub struct UnpreparedError {
    /// Unknown ID.
    pub id: CBytes,
}

impl FromCursor for UnpreparedError {
    fn from_cursor(mut cursor: &mut io::Cursor<&[u8]>) -> UnpreparedError {
        let id = CBytes::from_cursor(&mut cursor);

        UnpreparedError { id: id }
    }
}
