//! 错误类型定义
//!
//! 定义了应用的所有错误变体，每个错误对应一个唯一的错误码

use thiserror::Error;

/// 应用错误类型
///
/// 每个错误变体对应一个错误码，错误码格式为 6 位数字：MMTTNN
/// - MM (01-99): 模块代码
/// - TT (01-99): 类型代码
/// - NN (01-99): 具体代码
#[derive(Error, Debug)]
pub enum Error {
    // ==================== Common Errors (Auto Conversion) ====================
    /// 030000: IO error (file read/write, etc.)
    #[error("[030000] IO error")]
    Io(#[from] std::io::Error),

    /// 060503: JSON serialization/deserialization error
    #[error("[060503] Serialization error")]
    JsonSerialization(#[from] serde_json::Error),

    // ==================== Cache System (01) ====================
    /// 010101: Cache database creation failed
    #[error("[010101] Cache database creation failed")]
    CacheDbCreateFailed,

    /// 010102: Cache key derivation failed
    #[error("[010102] Cache key derivation failed")]
    CacheKeyDerivationFailed,

    /// 010103: Cache directory creation failed
    #[error("[010103] Cache directory creation failed")]
    CacheDirCreateFailed,

    /// 010104: Cache manager initialization failed
    #[error("[010104] Cache manager initialization failed")]
    CacheManagerInitFailed,

    /// 010401: Cache read failed
    #[error("[010401] Cache read failed")]
    CacheReadFailed,

    /// 010501: Cache write failed
    #[error("[010501] Cache write failed")]
    CacheWriteFailed,

    /// 010502: Cache delete failed
    #[error("[010502] Cache delete failed")]
    CacheDeleteFailed,

    /// 010503: Cache clear failed
    #[error("[010503] Cache clear failed")]
    CacheClearFailed,

    // ==================== Network Communication (02) ====================
    /// 020301: Server connection failed
    #[error("[020301] Server connection failed")]
    ServerConnectFailed,

    /// 020302: Server connection success
    #[error("[020302] Server connection success")]
    ServerConnectSuccess,

    /// 020303: Server disconnected
    #[error("[020303] Server disconnected")]
    ServerDisconnect,

    /// 020401: Network request failed
    #[error("[020401] Network request failed")]
    NetworkRequestFailed,

    /// 020801: Network request timeout
    #[error("[020801] Network request timeout")]
    NetworkTimeout,

    /// 020701: IP detection success
    #[error("[020701] IP detection success")]
    IpDetectSuccess,

    /// 020702: IP detection failed
    #[error("[020702] IP detection failed")]
    IpDetectFailed,

    // ==================== Storage Management (03) ====================
    /// 030101: App data directory access failed
    #[error("[030101] App data directory access failed")]
    AppDataDirFailed,

    /// 030102: Local directory creation failed
    #[error("[030102] Local directory creation failed")]
    LocalDirCreateFailed,

    /// 030103: Profiles directory creation failed
    #[error("[030103] Profiles directory creation failed")]
    ProfilesDirCreateFailed,

    /// 030104: Downloads directory creation failed
    #[error("[030104] Downloads directory creation failed")]
    DownloadsDirCreateFailed,

    /// 030105: Storage cache directory creation failed
    #[error("[030105] Storage cache directory creation failed")]
    StorageCacheDirCreateFailed,

    /// 030106: Logs directory creation failed
    #[error("[030106] Logs directory creation failed")]
    LogsDirCreateFailed,

    /// 030401: File read failed
    #[error("[030401] File read failed")]
    FileReadFailed,

    /// 030402: Directory read failed
    #[error("[030402] Directory read failed")]
    DirReadFailed,

    /// 030501: File write failed
    #[error("[030501] File write failed")]
    FileWriteFailed,

    /// 030502: File delete failed
    #[error("[030502] File delete failed")]
    FileDeleteFailed,

    // ==================== Authentication & Authorization (04) ====================
    /// 040101: Authentication initialization failed
    #[error("[040101] Authentication initialization failed")]
    AuthInitFailed,

    /// 040701: Credential validation failed
    #[error("[040701] Credential validation failed")]
    AuthCredentialInvalid,

    /// 040702: Token validation failed
    #[error("[040702] Token validation failed")]
    AuthTokenInvalid,

    /// 040703: Token expired
    #[error("[040703] Token expired")]
    AuthTokenExpired,

    /// 040801: Authentication timeout
    #[error("[040801] Authentication timeout")]
    AuthTimeout,

    // ==================== Device Fingerprint (05) ====================
    /// 050101: Device fingerprint initialization failed
    #[error("[050101] Device fingerprint initialization failed")]
    DeviceInitFailed,

    /// 050102: Machine code invalid
    #[error("[050102] Machine code invalid")]
    MachineCodeInvalid,

    /// 050401: Device info read failed
    #[error("[050401] Device info read failed")]
    DeviceInfoReadFailed,

    /// 050402: Machine code retrieval failed
    #[error("[050402] Machine code retrieval failed")]
    MachineCodeGetFailed,

    // ==================== Encryption & Decryption (06) ====================
    /// 060101: Crypto initialization failed
    #[error("[060101] Crypto initialization failed")]
    CryptoInitFailed,

    /// 060401: Decryption failed
    #[error("[060401] Decryption failed")]
    DecryptFailed,

    /// 060501: Encryption failed
    #[error("[060501] Encryption failed")]
    EncryptFailed,

    /// 060502: Key derivation failed
    #[error("[060502] Key derivation failed")]
    KeyDerivationFailed,

    /// 060504: Data deserialization failed
    #[error("[060504] Data deserialization failed")]
    DeserializeFailed,

    /// 060701: Public key parsing failed
    #[error("[060701] Public key parsing failed")]
    PublicKeyParseFailed,

    // ==================== Event System (07) ====================
    /// 070101: Event system initialization failed
    #[error("[070101] Event system initialization failed")]
    EventInitFailed,

    /// 070102: Event database creation failed
    #[error("[070102] Event database creation failed")]
    EventDbCreateFailed,

    /// 070301: Event sync failed
    #[error("[070301] Event sync failed")]
    EventSyncFailed,

    /// 070302: Event sync success
    #[error("[070302] Event sync success")]
    EventSyncSuccess,

    /// 070401: Event read failed
    #[error("[070401] Event read failed")]
    EventReadFailed,

    /// 070501: Event write failed
    #[error("[070501] Event write failed")]
    EventWriteFailed,

    /// 070502: Event delete failed
    #[error("[070502] Event delete failed")]
    EventDeleteFailed,

    // ==================== Browser Kernel (08) ====================
    /// 080101: Kernel initialization failed
    #[error("[080101] Kernel initialization failed")]
    KernelInitFailed,

    /// 080102: Kernel cache directory creation failed
    #[error("[080102] Kernel cache directory creation failed")]
    KernelCacheDirCreateFailed,

    /// 080301: Kernel download failed
    #[error("[080301] Kernel download failed")]
    KernelDownloadFailed,

    /// 080302: Kernel download success
    #[error("[080302] Kernel download success")]
    KernelDownloadSuccess,

    /// 080401: Kernel file read failed
    #[error("[080401] Kernel file read failed")]
    KernelFileReadFailed,

    /// 080501: Kernel installation failed
    #[error("[080501] Kernel installation failed")]
    KernelInstallFailed,

    /// 080601: Kernel preparation failed
    #[error("[080601] Kernel preparation failed: {0}")]
    KernelPrepareFailed(String),

    // ==================== Application Lifecycle (09) ====================
    /// 090101: Application initialization failed
    #[error("[090101] Application initialization failed")]
    AppInitFailed,

    /// 090102: Application start success
    #[error("[090102] Application start success")]
    AppStartSuccess,

    /// 090103: Application config load failed
    #[error("[090103] Application config load failed")]
    AppConfigLoadFailed,

    /// 091001: Application abnormal exit
    #[error("[091001] Application abnormal exit")]
    AppExitAbnormal,

    /// 091002: Application normal exit
    #[error("[091002] Application normal exit")]
    AppExitNormal,

    // ==================== Database Operations (10) ====================
    /// 100101: Database initialization failed
    #[error("[100101] Database initialization failed")]
    DbInitFailed,

    /// 100301: Database connection failed
    #[error("[100301] Database connection failed")]
    DbConnectFailed,

    /// 100302: Database connection success
    #[error("[100302] Database connection success")]
    DbConnectSuccess,

    /// 100401: Database query failed
    #[error("[100401] Database query failed")]
    DbQueryFailed,

    /// 100501: Database write failed
    #[error("[100501] Database write failed")]
    DbWriteFailed,

    /// 100502: Database update failed
    #[error("[100502] Database update failed")]
    DbUpdateFailed,

    /// 100503: Database delete failed
    #[error("[100503] Database delete failed")]
    DbDeleteFailed,

    // ==================== Update System (11) ====================
    /// 110101: Update check failed
    #[error("[110101] Update check failed: {0}")]
    UpdateCheckFailed(String),

    /// 110201: Update download failed
    #[error("[110201] Update download failed")]
    UpdateDownloadFailed,

    /// 110301: Update install failed
    #[error("[110301] Update install failed")]
    UpdateInstallFailed,

    /// 110401: Update manifest parse failed
    #[error("[110401] Update manifest parse failed")]
    UpdateManifestParseFailed,

    /// 110501: Update verification failed
    #[error("[110501] Update verification failed")]
    UpdateVerificationFailed,

    // ==================== Proxy System (12) ====================
    /// 120101: Proxy initialization failed
    #[error("[120101] Proxy initialization failed")]
    ProxyInitFailed,

    /// 120201: Proxy connection failed
    #[error("[120201] Proxy connection failed")]
    ProxyConnectionFailed,

    /// 120301: Proxy test failed
    #[error("[120301] Proxy test failed")]
    ProxyTestFailed,

    /// 120401: Proxy configuration invalid
    #[error("[120401] Proxy configuration invalid")]
    ProxyConfigInvalid,

    /// 120501: Proxy export failed
    #[error("[120501] Proxy export failed")]
    ProxyExportFailed,

    // ==================== Window Management (13) ====================
    /// 130101: Window creation failed
    #[error("[130101] Window creation failed")]
    WindowCreationFailed,

    /// 130201: Window operation failed
    #[error("[130201] Window operation failed")]
    WindowOperationFailed,

    /// 130301: Window not found
    #[error("[130301] Window not found")]
    WindowNotFound,

    /// 130401: Window state invalid
    #[error("[130401] Window state invalid")]
    WindowStateInvalid,

    /// 130501: Window layout calculation failed
    #[error("[130501] Window layout calculation failed")]
    WindowLayoutCalculationFailed,

    // ==================== Store/Configuration (14) ====================
    /// 140101: Store initialization failed
    #[error("[140101] Store initialization failed")]
    StoreInitFailed,

    /// 140201: Store read failed
    #[error("[140201] Store read failed")]
    StoreReadFailed,

    /// 140301: Store write failed
    #[error("[140301] Store write failed")]
    StoreWriteFailed,

    /// 140401: Store key not found
    #[error("[140401] Store key not found")]
    StoreKeyNotFound,

    /// 140501: Store parse failed
    #[error("[140501] Store parse failed")]
    StoreParseFailed,

    // ==================== Logging System (15) ====================
    /// 150101: Log initialization failed
    #[error("[150101] Log initialization failed")]
    LogInitFailed,

    /// 150201: Log write failed
    #[error("[150201] Log write failed")]
    LogWriteFailed,

    /// 150301: Log directory creation failed
    #[error("[150301] Log directory creation failed")]
    LogDirCreateFailed,

    // ==================== Process Management (16) ====================
    /// 160101: Process start failed
    #[error("[160101] Process start failed")]
    ProcessStartFailed,

    /// 160201: Process stop failed
    #[error("[160201] Process stop failed")]
    ProcessStopFailed,

    /// 160301: Process not found
    #[error("[160301] Process not found")]
    ProcessNotFound,

    /// 160401: Process kill failed
    #[error("[160401] Process kill failed")]
    ProcessKillFailed,

    // ==================== System Information (17) ====================
    /// 170101: System info retrieval failed
    #[error("[170101] System info retrieval failed")]
    SystemInfoFailed,

    /// 170201: Executable path retrieval failed
    #[error("[170201] Executable path retrieval failed")]
    ExecutablePathFailed,

    /// 170301: Environment variable access failed
    #[error("[170301] Environment variable access failed")]
    EnvVarAccessFailed,

    // ==================== Sync/Environment Management (18) ====================
    /// 180101: Sync start failed
    #[error("[180101] Sync start failed")]
    SyncStartFailed,

    /// 180201: Sync stop failed
    #[error("[180201] Sync stop failed")]
    SyncStopFailed,

    /// 180301: Environment launch failed
    #[error("[180301] Environment launch failed")]
    EnvironmentLaunchFailed,

    /// 180401: Environment stop failed
    #[error("[180401] Environment stop failed")]
    EnvironmentStopFailed,

    /// 180501: Environment not found
    #[error("[180501] Environment not found")]
    EnvironmentNotFound,

    // ==================== Configuration System (19) ====================
    /// 190101: Config initialization failed
    #[error("[190101] Config initialization failed: {0}")]
    ConfigInitFailed(String),

    /// 190201: Config load failed
    #[error("[190201] Config load failed: {0}")]
    ConfigLoadFailed(String),

    /// 190301: Config parse failed
    #[error("[190301] Config parse failed: {0}")]
    ConfigParseFailed(String),

    /// 190401: Config decrypt failed
    #[error("[190401] Config decrypt failed: {0}")]
    ConfigDecryptFailed(String),

    /// 190402: Config encrypt failed
    #[error("[190402] Config encrypt failed: {0}")]
    ConfigEncryptFailed(String),

    /// 190501: Config validation failed
    #[error("[190501] Config validation failed: {0}")]
    ConfigValidationFailed(String),

    /// 190601: Config already initialized
    #[error("[190601] Config already initialized")]
    ConfigAlreadyInitialized,

    /// 190701: Config not initialized
    #[error("[190701] Config not initialized")]
    ConfigNotInitialized,

    // ==================== Deep Link System (20) ====================
    /// 200101: Deep link initialization failed
    #[error("[200101] Deep link initialization failed")]
    DeepLinkInitFailed,

    /// 200201: Deep link parse failed
    #[error("[200201] Deep link parse failed")]
    DeepLinkParseFailed,

    /// 200301: Deep link registration failed
    #[error("[200301] Deep link registration failed")]
    DeepLinkRegistrationFailed,

    /// 200401: Deep link storage failed
    #[error("[200401] Deep link storage failed")]
    DeepLinkStorageFailed,

    // ==================== State Management (21) ====================
    /// 210101: State initialization failed
    #[error("[210101] State initialization failed")]
    StateInitFailed,

    /// 210201: State read failed
    #[error("[210201] State read failed")]
    StateReadFailed,

    /// 210301: State write failed
    #[error("[210301] State write failed")]
    StateWriteFailed,

    /// 210401: State lock failed
    #[error("[210401] State lock failed")]
    StateLockFailed,

    /// 210501: State invalid
    #[error("[210501] State invalid")]
    StateInvalid,

    // ==================== UI/Component System (22) ====================
    /// 220101: Tray initialization failed
    #[error("[220101] Tray initialization failed")]
    TrayInitFailed,

    /// 220201: Tray menu creation failed
    #[error("[220201] Tray menu creation failed")]
    TrayMenuCreationFailed,

    /// 220301: Tray icon load failed
    #[error("[220301] Tray icon load failed")]
    TrayIconLoadFailed,

    /// 220401: UI component initialization failed
    #[error("[220401] UI component initialization failed")]
    UiComponentInitFailed,

    // ==================== Path Management (23) ====================
    /// 230101: Path resolution failed
    #[error("[230101] Path resolution failed")]
    PathResolutionFailed,

    /// 230201: Path validation failed
    #[error("[230201] Path validation failed")]
    PathValidationFailed,

    /// 230301: Path creation failed
    #[error("[230301] Path creation failed")]
    PathCreationFailed,

    /// 230401: Path not found
    #[error("[230401] Path not found")]
    PathNotFound,

    // ==================== Request/HTTP Client (24) ====================
    /// 240101: HTTP client initialization failed
    #[error("[240101] HTTP client initialization failed")]
    HttpClientInitFailed,

    /// 240201: HTTP request build failed
    #[error("[240201] HTTP request build failed")]
    HttpRequestBuildFailed,

    /// 240301: HTTP response parse failed
    #[error("[240301] HTTP response parse failed")]
    HttpResponseParseFailed,

    /// 240401: HTTP interceptor failed
    #[error("[240401] HTTP interceptor failed")]
    HttpInterceptorFailed,

    // ==================== Anchor/Secure Storage (25) ====================
    /// 250101: Anchor initialization failed
    #[error("[250101] Anchor initialization failed")]
    AnchorInitFailed,

    /// 250201: Anchor read failed
    #[error("[250201] Anchor read failed")]
    AnchorReadFailed,

    /// 250301: Anchor write failed
    #[error("[250301] Anchor write failed")]
    AnchorWriteFailed,

    /// 250401: Anchor encryption failed
    #[error("[250401] Anchor encryption failed")]
    AnchorEncryptionFailed,

    /// 250501: Anchor decryption failed
    #[error("[250501] Anchor decryption failed")]
    AnchorDecryptionFailed,

    // ==================== Credential Management (26) ====================
    /// 260101: Credential initialization failed
    #[error("[260101] Credential initialization failed")]
    CredentialInitFailed,

    /// 260201: Credential save failed
    #[error("[260201] Credential save failed")]
    CredentialSaveFailed,

    /// 260301: Credential load failed
    #[error("[260301] Credential load failed")]
    CredentialLoadFailed,

    /// 260401: Credential delete failed
    #[error("[260401] Credential delete failed")]
    CredentialDeleteFailed,

    /// 260501: Credential encryption failed
    #[error("[260501] Credential encryption failed")]
    CredentialEncryptionFailed,

    // ==================== Platform/System Detection (27) ====================
    /// 270101: Platform detection failed
    #[error("[270101] Platform detection failed")]
    PlatformDetectionFailed,

    /// 270201: System version retrieval failed
    #[error("[270201] System version retrieval failed")]
    SystemVersionFailed,

    /// 270301: Hardware info retrieval failed
    #[error("[270301] Hardware info retrieval failed")]
    HardwareInfoFailed,

    /// 270401: TPM access failed
    #[error("[270401] TPM access failed")]
    TpmAccessFailed,

    /// 270501: SMBIOS read failed
    #[error("[270501] SMBIOS read failed")]
    SmbiosReadFailed,

    // ==================== Validation/Verification (28) ====================
    /// 280101: Validation failed
    #[error("[280101] Validation failed")]
    ValidationFailed,

    /// 280201: Verification failed
    #[error("[280201] Verification failed")]
    VerificationFailed,

    /// 280301: Checksum mismatch
    #[error("[280301] Checksum mismatch")]
    ChecksumMismatch,

    /// 280401: Signature verification failed
    #[error("[280401] Signature verification failed")]
    SignatureVerificationFailed,

    /// 280501: Integrity check failed
    #[error("[280501] Integrity check failed")]
    IntegrityCheckFailed,

    // ==================== Permission/Access Control (29) ====================
    /// 290101: Permission denied
    #[error("[290101] Permission denied")]
    PermissionDenied,

    /// 290201: Access denied
    #[error("[290201] Access denied")]
    AccessDenied,

    /// 290301: Insufficient privileges
    #[error("[290301] Insufficient privileges")]
    InsufficientPrivileges,

    /// 290401: Resource locked
    #[error("[290401] Resource locked")]
    ResourceLocked,

    // ==================== General/Utility Errors (99) ====================
    /// 990101: Operation not supported
    #[error("[990101] Operation not supported")]
    OperationNotSupported,

    /// 990201: Invalid argument
    #[error("[990201] Invalid argument")]
    InvalidArgument,

    /// 990301: Invalid state
    #[error("[990301] Invalid state")]
    InvalidState,

    /// 990401: Timeout
    #[error("[990401] Timeout")]
    Timeout,

    /// 990501: Resource not found
    #[error("[990501] Resource not found")]
    ResourceNotFound,

    /// 990601: Resource already exists
    #[error("[990601] Resource already exists")]
    ResourceAlreadyExists,

    /// 990701: Operation cancelled
    #[error("[990701] Operation cancelled")]
    OperationCancelled,

    /// 990801: Internal error
    #[error("[990801] Internal error")]
    InternalError,

    /// 990901: Unknown error
    #[error("[990901] Unknown error")]
    UnknownError,
}

impl Error {
    /// 提取错误码
    ///
    /// 从错误信息中解析出 6 位数字错误码
    pub fn code(&self) -> String {
        let msg = self.to_string();
        if let Some(start) = msg.find('[') {
            if let Some(end) = msg.find(']') {
                return msg[start + 1..end].to_string();
            }
        }
        "000000".to_string()
    }

    /// 记录错误日志（环境感知）
    ///
    /// - 开发环境：记录完整错误信息
    /// - 生产环境：只记录错误码
    ///
    /// # 示例
    /// ```
    /// return Err(Error::CacheReadFailed.log());
    /// ```
    pub fn log(self) -> Self {
        if cfg!(debug_assertions) {
            log::error!("[{}] {}", self.code(), self);
        } else {
            log::error!("[{}]", self.code());
        }
        self
    }

    /// 记录错误日志并附加上下文（环境感知）
    ///
    /// - 开发环境：记录错误信息和上下文
    /// - 生产环境：只记录错误码
    ///
    /// # 示例
    /// ```
    /// return Err(Error::CacheReadFailed.log_with(format!("key: {}", key)));
    /// ```
    pub fn log_with(self, context: impl std::fmt::Display) -> Self {
        if cfg!(debug_assertions) {
            log::error!("[{}] {}: {}", self.code(), self, context);
        } else {
            log::error!("[{}]", self.code());
        }
        self
    }

    /// 记录警告日志（环境感知）
    pub fn log_warn(self) -> Self {
        if cfg!(debug_assertions) {
            log::warn!("[{}] {}", self.code(), self);
        } else {
            log::warn!("[{}]", self.code());
        }
        self
    }

    /// 记录警告日志并附加上下文（环境感知）
    pub fn log_warn_with(self, context: impl std::fmt::Display) -> Self {
        if cfg!(debug_assertions) {
            log::warn!("[{}] {}: {}", self.code(), self, context);
        } else {
            log::warn!("[{}]", self.code());
        }
        self
    }

    /// 格式化为前端错误信息
    ///
    /// - 开发环境：返回完整错误信息（错误码 + 描述）
    /// - 生产环境：只返回错误码
    pub(super) fn format_for_frontend(&self) -> String {
        if cfg!(debug_assertions) {
            self.to_string()
        } else {
            format!("[{}]", self.code())
        }
    }
}
