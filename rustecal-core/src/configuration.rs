//! configuration.rs
//!
//! Provides a safe Rust wrapper around the eCAL C Configuration API.
//!
//! This module exposes a `Configuration` struct that manages an
//! `eCAL_Configuration` instance via FFI. It supports initializing
//! default settings or loading from a YAML file, and automatically
//! frees the underlying C object on drop.

use rustecal_sys as sys;
use std::{
    ffi::{CStr, CString},
    ops::{Deref, DerefMut},
    path::Path,
};
use thiserror::Error;

/// Errors that can occur when creating or loading a Configuration
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Received null pointer from eCAL_Configuration_New")]
    NullPointer,
    #[error("Invalid file path: {0}")]
    InvalidPath(String),
}

/// Safe Rust wrapper around eCAL_Configuration
pub struct Configuration {
    inner: *mut sys::eCAL_Configuration,
}

unsafe impl Send for Configuration {}
unsafe impl Sync for Configuration {}

impl Configuration {
    /// Creates a new Configuration with default values loaded via eCAL_Configuration_InitFromConfig
    pub fn new() -> Result<Self, ConfigError> {
        let cfg = unsafe { sys::eCAL_Configuration_New() };
        if cfg.is_null() {
            return Err(ConfigError::NullPointer);
        }
        unsafe { sys::eCAL_Configuration_InitFromConfig(cfg) };
        Ok(Configuration { inner: cfg })
    }

    /// Loads a Configuration from a YAML file at the given path
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        if !Path::new(path).exists() {
            return Err(ConfigError::InvalidPath(path.to_string()));
        }
        let c_path = CString::new(path).map_err(|_| ConfigError::InvalidPath(path.to_string()))?;
        let cfg = unsafe { sys::eCAL_Configuration_New() };
        if cfg.is_null() {
            return Err(ConfigError::NullPointer);
        }
        unsafe { sys::eCAL_Configuration_InitFromFile(cfg, c_path.as_ptr()) };
        Ok(Configuration { inner: cfg })
    }

    /// Returns the path of the loaded configuration file, if any
    pub fn file_path(&self) -> Option<String> {
        unsafe {
            let p = sys::eCAL_Configuration_GetConfigurationFilePath(self.inner);
            if p.is_null() {
                None
            } else {
                Some(CStr::from_ptr(p).to_string_lossy().into_owned())
            }
        }
    }

    /// Returns a raw pointer to the underlying eCAL_Configuration for FFI calls
    pub(crate) fn as_ptr(&self) -> *const sys::eCAL_Configuration {
        self.inner as *const _
    }
}

/// Allow transparent access to the underlying C struct
impl Deref for Configuration {
    type Target = sys::eCAL_Configuration;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}

/// Allow mutable access to the underlying C struct
impl DerefMut for Configuration {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner }
    }
}

impl Drop for Configuration {
    /// Deletes the underlying eCAL_Configuration on drop
    fn drop(&mut self) {
        unsafe { sys::eCAL_Configuration_Delete(self.inner) };
    }
}
