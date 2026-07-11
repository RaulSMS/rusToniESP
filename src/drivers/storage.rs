use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageError {
    MountFailed,
    UnmountFailed,
    NotMounted,
    AlreadyMounted,
    IoError,
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::MountFailed => write!(f, "Failed to mount storage"),
            StorageError::UnmountFailed => write!(f, "Failed to unmount storage"),
            StorageError::NotMounted => write!(f, "Storage is not mounted"),
            StorageError::AlreadyMounted => write!(f, "Storage is already mounted"),
            StorageError::IoError => write!(f, "I/O error occurred on storage"),
        }
    }
}

impl std::error::Error for StorageError {}

pub trait StorageDriver {
    /// Mounts the SD Card filesystem.
    fn mount(&self) -> Result<(), StorageError>;
    
    /// Unmounts the PDF/SD Card filesystem.
    fn unmount(&self) -> Result<(), StorageError>;
    
    /// Returns the mount point path (e.g. "/sdcard").
    fn mount_point(&self) -> &str;
    
    /// Checks if the storage media is mounted.
    fn is_mounted(&self) -> bool;
}

use std::sync::Mutex;

pub struct MockStorageDriver {
    mounted: Mutex<bool>,
    mount_point: String,
}

impl MockStorageDriver {
    pub fn new(mount_point: impl Into<String>) -> Self {
        Self {
            mounted: Mutex::new(false),
            mount_point: mount_point.into(),
        }
    }
}

pub struct SpiSdCardDriver {
    mounted: Mutex<bool>,
    mount_point: String,
}

impl SpiSdCardDriver {
    pub fn new(mount_point: impl Into<String>) -> Self {
        Self {
            mounted: Mutex::new(false),
            mount_point: mount_point.into(),
        }
    }
}

impl StorageDriver for MockStorageDriver {
    fn mount(&self) -> Result<(), StorageError> {
        let mut mounted = self.mounted.lock().unwrap();
        if *mounted {
            return Err(StorageError::AlreadyMounted);
        }

        std::fs::create_dir_all(&self.mount_point).map_err(|_| StorageError::MountFailed)?;
        *mounted = true;
        Ok(())
    }

    fn unmount(&self) -> Result<(), StorageError> {
        let mut mounted = self.mounted.lock().unwrap();
        if !*mounted {
            return Err(StorageError::NotMounted);
        }
        *mounted = false;
        Ok(())
    }

    fn mount_point(&self) -> &str {
        &self.mount_point
    }

    fn is_mounted(&self) -> bool {
        *self.mounted.lock().unwrap()
    }
}

impl StorageDriver for SpiSdCardDriver {
    fn mount(&self) -> Result<(), StorageError> {
        let mut mounted = self.mounted.lock().unwrap();
        if *mounted {
            return Err(StorageError::AlreadyMounted);
        }

        #[cfg(not(target_os = "espidf"))]
        {
            std::fs::create_dir_all(&self.mount_point).map_err(|_| StorageError::MountFailed)?;
        }

        #[cfg(target_os = "espidf")]
        {
            let _ = self.mount_point();
        }

        *mounted = true;
        Ok(())
    }

    fn unmount(&self) -> Result<(), StorageError> {
        let mut mounted = self.mounted.lock().unwrap();
        if !*mounted {
            return Err(StorageError::NotMounted);
        }
        *mounted = false;
        Ok(())
    }

    fn mount_point(&self) -> &str {
        &self.mount_point
    }

    fn is_mounted(&self) -> bool {
        *self.mounted.lock().unwrap()
    }
}

