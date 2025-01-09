#![doc = include_str!("../README.md")]

pub mod error;
pub mod prelude;

use crate::prelude::*;

use std::ffi::{CStr, CString};
use std::path::Path;

use bitflags::bitflags;

bitflags! {
    /// Represents various flags for configuring scan operations.
    ///
    /// The `ScanFlags` structure contains constants that can be used to specify
    /// the behavior of scanning processes. Each flag can be combined using bitwise
    /// operations to enable multiple options at once.
    pub struct ScanFlags: u32 {
        /// Enables a thorough scanning process that examines all files and directories
        /// in detail.
        const DEEP_SCAN = 0x00000001;
        /// Activates heuristic analysis to detect potential threats based on behavior
        /// rather than signatures.
        const HEURISTIC_SCAN = 0x00000002;
        /// Allows scanning of all file types, regardless of their extensions or formats.
        const ALLTYPES_SCAN = 0x00000004;
        /// Enables scanning of all subdirectories within the target directory, not just
        /// the top-level files.
        const RECURSIVE_SCAN = 0x00000008;
        /// Provides detailed output during the scanning process, useful for debugging
        /// or monitoring.
        const VERBOSE = 0x00000010;
        /// Formats the scan results as XML.
        const RESULT_AS_XML = 0x00010000;
        /// Formats the scan results as JSON.
        const RESULT_AS_JSON = 0x00020000;
        /// Formats the scan results as TSV (Tab-Separated Values).
        const RESULT_AS_TSV = 0x00040000;
        /// Formats the scan results as CSV (Comma-Separated Values).
        const RESULT_AS_CSV = 0x00080000;
    }
}

#[link(name = "die", kind = "static")]
unsafe extern "C" {
    // Definitions from die.h
    fn DIE_FreeMemoryA(str: *const i8);
    fn DIE_ScanFileA(fname: *const i8, flags: u32, db: *const i8) -> *const i8;
    fn DIE_ScanFileExA(fname: *const i8, flags: u32) -> *mut i8;
    fn DIE_ScanMemoryA(mem: *const u8, len: u32, flags: u32, db: *const i8) -> *mut i8;
    fn DIE_ScanMemoryExA(mem: *const u8, len: u32, flags: u32) -> *mut i8;
    fn DIE_LoadDatabaseA(fname: *const i8) -> i32;
}

/// Scans a file at the specified path using the provided scan flags.
///
/// # Description
///
/// This function performs a scan on the file located at `fpath`. The behavior of the scan
/// can be customized using the `flags` parameter, which allows for various scanning options
/// such as deep scanning, heuristic analysis, and result formatting.
///
/// # Parameters
///
/// - `fpath`: A reference to a `Path` object representing the file path to be scanned.
/// - `flags`: An instance of `ScanFlags` that specifies the scanning options to be applied.
///
/// # Returns
///
/// - `Result<String>`: On success, returns a `String` containing the scan results. If the scan
///   fails, it returns an appropriate error wrapped in a `Result`.
///
/// # Example
///
/// ```rust
/// # use std::path::Path;
/// use die::{scan_file, ScanFlags};
/// let path = Path::new("example.txt");
/// let flags = ScanFlags::DEEP_SCAN | ScanFlags::VERBOSE;
/// match scan_file(&path, flags) {
///     Ok(results) => println!("Scan results: {}", results),
///     Err(e) => eprintln!("Error scanning file: {}", e),
/// }
/// ```
///
pub fn scan_file(fpath: &Path, flags: ScanFlags) -> Result<String> {
    let fpath = CString::new(fpath.to_str().ok_or(Error::ConversionFailure)?)?;

    unsafe {
        let res = DIE_ScanFileExA(fpath.as_ptr(), flags.bits());
        let out = CStr::from_ptr(res).to_str()?.to_string();
        DIE_FreeMemoryA(res);
        Ok(out)
    }
}

/// Scans a file at the specified path using the provided scan flags and a database.
///
/// # Description
/// This function performs a scan on the file located at `fpath`, utilizing the specified
/// `flags` for customizing the scan behavior. Additionally, it uses a database located at
/// `db_path` to enhance the scanning process, potentially allowing for more accurate or
/// efficient results.
///
/// # Parameters
///
/// - `fpath`: A reference to a `Path` object representing the file path to be scanned.
/// - `flags`: An instance of `ScanFlags` that specifies the scanning options to be applied.
/// - `db_path`: A reference to a `Path` object representing the location of the database
///   used during the scanning process.
///
/// # Returns
/// - `Result<String>`: On success, returns a `String` containing the scan results. If the scan
///   fails, it returns an appropriate error wrapped in a `Result`.
///
/// # Example
/// ```rust
/// # use std::path::Path;
/// use die::{scan_file_with_db, ScanFlags};
/// let path = Path::new("example.txt");
/// let db_path = Path::new("/path/to/die/database.db");
/// let flags = ScanFlags::DEEP_SCAN | ScanFlags::VERBOSE;
/// match scan_file_with_db(&path, flags, &db_path) {
///     Ok(results) => println!("Scan results: {}", results),
///     Err(e) => eprintln!("Error scanning file: {}", e),
/// }
/// ```
///
pub fn scan_file_with_db(fpath: &Path, flags: ScanFlags, db_path: &Path) -> Result<String> {
    let fpath = CString::new(fpath.to_str().ok_or(Error::ConversionFailure)?)?;
    let db_path = CString::new(db_path.to_str().ok_or(Error::ConversionFailure)?)?;

    unsafe {
        let cstr = CStr::from_ptr(fpath.as_ptr() as *const i8);
        let res = DIE_ScanFileA(cstr.as_ptr(), flags.bits(), db_path.as_ptr());
        let str = CStr::from_ptr(res).to_str()?.to_string();
        DIE_FreeMemoryA(res);
        Ok(str)
    }
}

/// Scans a byte array in memory using the provided scan flags.
///
/// # Description
///
/// This function performs a scan on the data contained in the `mem` byte slice. The behavior of the scan
/// can be customized using the `flags` parameter, which allows for various scanning options such as deep
/// scanning and heuristic analysis.
///
/// # Parameters
/// - `mem`: A reference to a byte slice (`&[u8]`) representing the in-memory data to be scanned.
/// - `flags`: An instance of `ScanFlags` that specifies the scanning options to be applied.
///
/// # Returns
/// - `Result<String>`: On success, returns a `String` containing the scan results. If the scan
///   fails, it returns an appropriate error wrapped in a `Result`.
///
/// # Example
/// ```rust
/// # use std::path::Path;
/// use die::{scan_memory, ScanFlags};
/// let data: &[u8] = &[0x00, 0x01, 0x02, 0x03];
/// let flags = ScanFlags::HEURISTIC_SCAN | ScanFlags::VERBOSE;
/// match scan_memory(data, flags) {
///     Ok(results) => println!("Scan results: {}", results),
///     Err(e) => eprintln!("Error scanning memory: {}", e),
/// }
/// ```
pub fn scan_memory(mem: &[u8], flags: ScanFlags) -> Result<String> {
    let ptr = mem.as_ptr();
    let sz = mem.len();

    if sz > u32::MAX as usize {
        return Err(Error::Overflow);
    }

    unsafe {
        let res = DIE_ScanMemoryExA(ptr, sz as u32, flags.bits());
        let str = CStr::from_ptr(res).to_str()?.to_string();
        DIE_FreeMemoryA(res);
        Ok(str)
    }
}

/// Scans a byte array in memory using the provided scan flags and a database.
///
/// # Description
/// This function performs a scan on the data contained in the `mem` byte slice, utilizing the specified
/// `flags` for customizing the scan behavior. Additionally, it uses a database located at `db_path`
/// to enhance the scanning process, potentially allowing for more accurate or efficient results.
///
/// # Parameters
/// - `mem`: A reference to a byte slice (`&[u8]`) representing the in-memory data to be scanned.
/// - `flags`: An instance of `ScanFlags` that specifies the scanning options to be applied.
/// - `db_path`: A reference to a `Path` object representing the location of the database
///   used during the scanning process.
///
/// # Returns
/// - `Result<String>`: On success, returns a `String` containing the scan results. If the scan
///   fails, it returns an appropriate error wrapped in a `Result`.
///
/// # Example
/// ```rust
/// # use std::path::Path;
/// use die::{scan_memory_with_db, ScanFlags};
/// let path = Path::new("example.txt");
/// let db_path = Path::new("/path/to/die/database.db");
/// let flags = ScanFlags::DEEP_SCAN | ScanFlags::VERBOSE;
/// let data: &[u8] = &[0x00, 0x01, 0x02, 0x03];
/// match scan_memory_with_db(data, flags, &db_path) {
///     Ok(results) => println!("Scan results: {}", results),
///     Err(e) => eprintln!("Error scanning memory: {}", e),
/// }
/// ```
pub fn scan_memory_with_db(mem: &[u8], flags: ScanFlags, db_path: &Path) -> Result<String> {
    let ptr = mem.as_ptr();
    let sz = mem.len();
    let db_path = CString::new(db_path.to_str().ok_or(Error::ConversionFailure)?)?;

    if sz > u32::MAX as usize {
        return Err(Error::Overflow);
    }

    unsafe {
        let res = DIE_ScanMemoryA(ptr, sz as u32, flags.bits(), db_path.as_ptr());
        let str = CStr::from_ptr(res).to_str()?.to_string();
        DIE_FreeMemoryA(res);
        Ok(str)
    }
}

/// Loads the database from the specified file path.
///
/// # Description
///
/// This function attempts to load a database from the given file path and returns
/// a result indicating the success or failure of the operation. On success, it returns
/// an integer representing the database identifier.
///
/// # Arguments
///
/// * `fpath` - A reference to a `Path` that specifies the location of the database file.
///
/// # Returns
///
/// * `Result<()>` - On success, returns `Ok()`.
///                  On failure, returns an `Err` with the error details.
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// use die::load_database;
///
/// let path = Path::new("/path/to/die/database.db");
/// match load_database(&path) {
///     Ok(_) => println!("Database path successfully loaded"),
///     Err(e) => println!("Failed to load database: {:?}", e),
/// }
/// ```
pub fn load_database(fpath: &Path) -> Result<()> {
    let fpath = CString::new(fpath.to_str().ok_or(Error::ConversionFailure)?)?;

    let res = unsafe { DIE_LoadDatabaseA(fpath.as_ptr()) };
    match res {
        0 => Ok(()),
        err => Err(Error::Ffi { error_code: err }),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use std::fs::File;
    use std::path::Path;

    use memmap2::Mmap;

    fn default_test_file() -> &'static Path {
        if cfg!(not(target_os = "windows")) {
            Path::new("/bin/ls")
        } else {
            Path::new("c:/windows/system32/winver.exe")
        }
    }

    fn default_test_file_type() -> &'static str {
        if cfg!(target_os = "linux") {
            "ELF64"
        } else if cfg!(target_os = "macos") {
            "MACHO64"
        } else {
            "PE64"
        }
    }

    #[test]
    fn test_scan_file() {
        let fname = default_test_file();
        let expected_type = default_test_file_type();

        let flags = ScanFlags::DEEP_SCAN;
        let res = scan_file(fname, flags).unwrap();
        assert!(
            res.starts_with(expected_type),
            "unexpected result: {:?}",
            res
        );
    }

    #[test]
    fn test_scan_file_db() {
        let fname = default_test_file();
        let expected_type = default_test_file_type();

        let flags = ScanFlags::DEEP_SCAN;
        if let Ok(db_path) = std::env::var("DIE_DB_PATH") {
            let res = scan_file_with_db(&fname, flags, Path::new(&db_path)).unwrap();
            assert!(
                res.starts_with(expected_type),
                "unexpected result: {:?}",
                res
            );
        } else {
            println!("Missing `DIE_DB_PATH` env var, skipping test");
        }
    }

    #[test]
    fn test_scan_memory() {
        let fname = default_test_file();
        let expected_type = default_test_file_type();

        let flags = ScanFlags::DEEP_SCAN;
        let file = File::open(fname).unwrap();
        let mem = unsafe { Mmap::map(&file).unwrap() };

        let res = scan_memory(mem.as_ref(), flags).unwrap();
        assert!(
            res.starts_with(expected_type),
            "unexpected result: {:?}",
            res
        );
    }

    #[test]
    fn test_scan_memory_with_db() {
        let fname = default_test_file();
        let expected_type = default_test_file_type();

        let flags = ScanFlags::DEEP_SCAN;
        let file = File::open(fname).unwrap();
        let mem = unsafe { Mmap::map(&file).unwrap() };

        if let Ok(db_path) = std::env::var("DIE_DB_PATH") {
            let res = scan_memory_with_db(mem.as_ref(), flags, Path::new(&db_path)).unwrap();
            assert!(
                res.starts_with(expected_type),
                "unexpected result: {:?}",
                res
            );
        } else {
            println!("Missing `DIE_DB_PATH` env var, skipping test");
        }
    }
}
