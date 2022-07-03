use std::error::Error;
use std::fmt::{Display, Formatter};

pub(crate) mod db;
pub(crate) mod tx;

#[cxx::bridge]
pub(crate) mod ffi {
    #[derive(Debug)]
    pub struct DbOpts<'a> {
        pub db_path: &'a str,
        pub optimistic: bool,
        pub prepare_for_bulk_load: bool,
        pub increase_parallelism: usize,
        pub optimize_level_style_compaction: bool,
        pub create_if_missing: bool,
        pub paranoid_checks: bool,
        pub enable_blob_files: bool,
        pub min_blob_size: usize,
        pub blob_file_size: usize,
        pub enable_blob_garbage_collection: bool,
        pub use_bloom_filter: bool,
        pub bloom_filter_bits_per_key: f64,
        pub bloom_filter_whole_key_filtering: bool,
        pub use_capped_prefix_extractor: bool,
        pub capped_prefix_extractor_len: usize,
        pub use_fixed_prefix_extractor: bool,
        pub fixed_prefix_extractor_len: usize,
        pub comparator_impl: *const u8,
        pub comparator_name: &'a str,
        pub comparator_different_bytes_can_be_equal: bool,
        pub destroy_on_exit: bool,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct RdbStatus {
        pub code: StatusCode,
        pub subcode: StatusSubCode,
        pub severity: StatusSeverity,
        pub message: String,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum StatusCode {
        kOk = 0,
        kNotFound = 1,
        kCorruption = 2,
        kNotSupported = 3,
        kInvalidArgument = 4,
        kIOError = 5,
        kMergeInProgress = 6,
        kIncomplete = 7,
        kShutdownInProgress = 8,
        kTimedOut = 9,
        kAborted = 10,
        kBusy = 11,
        kExpired = 12,
        kTryAgain = 13,
        kCompactionTooLarge = 14,
        kColumnFamilyDropped = 15,
        kMaxCode,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum StatusSubCode {
        kNone = 0,
        kMutexTimeout = 1,
        kLockTimeout = 2,
        kLockLimit = 3,
        kNoSpace = 4,
        kDeadlock = 5,
        kStaleFile = 6,
        kMemoryLimit = 7,
        kSpaceLimit = 8,
        kPathNotFound = 9,
        KMergeOperandsInsufficientCapacity = 10,
        kManualCompactionPaused = 11,
        kOverwritten = 12,
        kTxnNotPrepared = 13,
        kIOFenced = 14,
        kMaxSubCode,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum StatusSeverity {
        kNoError = 0,
        kSoftError = 1,
        kHardError = 2,
        kFatalError = 3,
        kUnrecoverableError = 4,
        kMaxSeverity,
    }

    unsafe extern "C++" {
        include!("bridge.h");

        type StatusCode;
        type StatusSubCode;
        type StatusSeverity;
        type WriteOptions;
        type PinnableSlice;

        fn set_w_opts_sync(o: Pin<&mut WriteOptions>, val: bool);
        fn set_w_opts_disable_wal(o: Pin<&mut WriteOptions>, val: bool);
        fn set_w_opts_no_slowdown(o: Pin<&mut WriteOptions>, val: bool);

        type ReadOptions;

        type RocksDbBridge;
        fn get_db_path(self: &RocksDbBridge) -> &CxxString;
        fn open_db(builder: &DbOpts, status: &mut RdbStatus) -> SharedPtr<RocksDbBridge>;
        fn transact(self: &RocksDbBridge) -> UniquePtr<TxBridge>;

        type TxBridge;
        fn get_w_opts(self: Pin<&mut TxBridge>) -> Pin<&mut WriteOptions>;
        fn set_snapshot(self: Pin<&mut TxBridge>);
        fn clear_snapshot(self: Pin<&mut TxBridge>);
        fn get(
            self: Pin<&mut TxBridge>,
            key: &[u8],
            for_update: bool,
            status: &mut RdbStatus,
        ) -> UniquePtr<PinnableSlice>;
        fn put(self: Pin<&mut TxBridge>, key: &[u8], val: &[u8], status: &mut RdbStatus);
        fn del(self: Pin<&mut TxBridge>, key: &[u8], status: &mut RdbStatus);
        fn commit(self: Pin<&mut TxBridge>, status: &mut RdbStatus);
        fn rollback(self: Pin<&mut TxBridge>, status: &mut RdbStatus);
        fn rollback_to_savepoint(self: Pin<&mut TxBridge>, status: &mut RdbStatus);
        fn pop_savepoint(self: Pin<&mut TxBridge>, status: &mut RdbStatus);
    }
}

impl Default for ffi::RdbStatus {
    fn default() -> Self {
        ffi::RdbStatus {
            code: ffi::StatusCode::kOk,
            subcode: ffi::StatusSubCode::kNone,
            severity: ffi::StatusSeverity::kNoError,
            message: "".to_string(),
        }
    }
}

impl Error for ffi::RdbStatus {}

impl Display for ffi::RdbStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.message.is_empty() {
            write!(f, "RocksDB error: {:?}", self)
        } else {
            write!(f, "RocksDB error: {}", self.message)
        }
    }
}

impl ffi::RdbStatus {
    #[inline(always)]
    pub fn is_ok(&self) -> bool {
        self.code == ffi::StatusCode::kOk
    }
    #[inline(always)]
    pub fn is_not_found(&self) -> bool {
        self.code == ffi::StatusCode::kNotFound
    }
    #[inline(always)]
    pub fn is_ok_or_not_found(&self) -> bool {
        self.is_ok() || self.is_not_found()
    }
}
