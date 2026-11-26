use std::ffi::{CStr, CString};
use std::ops::Deref;

use crocksdb_ffi::{self, DBCompressionType, DBTitanBlobIndex, DBTitanDBOptions};
use librocksdb_sys::{ctitandb_encode_blob_index, DBTitanDBBlobRunMode};
use rocksdb::Cache;
use std::ops::DerefMut;
use std::ptr;
use std::slice;

pub struct TitanDBOptions {
    pub(crate) inner: *mut DBTitanDBOptions,
}

impl TitanDBOptions {
    pub fn new() -> Self {
        unsafe {
            Self {
                inner: crocksdb_ffi::ctitandb_options_create(),
            }
        }
    }

    pub fn dirname(&self) -> &str {
        unsafe {
            let name = crocksdb_ffi::ctitandb_options_dirname(self.inner);
            CStr::from_ptr(name).to_str().unwrap()
        }
    }

    pub fn set_dirname(&mut self, name: &str) {
        let s = CString::new(name).unwrap();
        // Safety: set_dirname copies the C string into std::string. We
        // still own s and must drop it.
        unsafe {
            crocksdb_ffi::ctitandb_options_set_dirname(self.inner, s.as_ptr());
        }
    }

    pub fn min_blob_size(&self) -> u64 {
        unsafe { crocksdb_ffi::ctitandb_options_min_blob_size(self.inner) }
    }

    pub fn set_min_blob_size(&mut self, size: u64) {
        unsafe {
            crocksdb_ffi::ctitandb_options_set_min_blob_size(self.inner, size);
        }
    }

    /// Enables version 2 blob indices and backfills legacy indices during SST
    /// construction. Requires reopening the DB. Existing version 2 indices
    /// prevent downgrade to older readers even after this option is disabled.
    pub fn set_enable_blob_index_raw_size(&mut self, enable: bool) {
        unsafe {
            crocksdb_ffi::ctitandb_options_set_enable_blob_index_raw_size(self.inner, enable);
        }
    }

    pub fn enable_blob_index_raw_size(&self) -> bool {
        unsafe { crocksdb_ffi::ctitandb_options_enable_blob_index_raw_size(self.inner) }
    }

    pub fn blob_file_compression(&self) -> DBCompressionType {
        unsafe { crocksdb_ffi::ctitandb_options_blob_file_compression(self.inner) }
    }

    pub fn set_blob_file_compression(&mut self, t: DBCompressionType) {
        unsafe {
            crocksdb_ffi::ctitandb_options_set_blob_file_compression(self.inner, t);
        }
    }

    pub fn set_compression_options(
        &mut self,
        window_bits: i32,
        level: i32,
        strategy: i32,
        max_dict_bytes: i32,
        zstd_max_train_bytes: i32,
    ) {
        unsafe {
            crocksdb_ffi::ctitandb_options_set_compression_options(
                self.inner,
                window_bits,
                level,
                strategy,
                max_dict_bytes,
                zstd_max_train_bytes,
            )
        }
    }

    pub fn set_disable_background_gc(&mut self, disable: bool) {
        unsafe {
            crocksdb_ffi::ctitandb_options_set_disable_background_gc(self.inner, disable);
        }
    }

    pub fn set_level_merge(&mut self, enable: bool) {
        unsafe {
            crocksdb_ffi::ctitandb_options_set_level_merge(self.inner, enable);
        }
    }

    pub fn set_range_merge(&mut self, enable: bool) {
        unsafe {
            crocksdb_ffi::ctitandb_options_set_range_merge(self.inner, enable);
        }
    }

    pub fn set_max_sorted_runs(&mut self, size: i32) {
        unsafe {
            crocksdb_ffi::ctitandb_options_set_max_sorted_runs(self.inner, size);
        }
    }

    pub fn set_max_background_gc(&mut self, size: i32) {
        unsafe {
            crocksdb_ffi::ctitandb_options_set_max_background_gc(self.inner, size);
        }
    }

    pub fn set_purge_obsolete_files_period(&mut self, period: usize) {
        unsafe {
            crocksdb_ffi::ctitandb_options_set_purge_obsolete_files_period_sec(self.inner, period);
        }
    }

    pub fn set_min_gc_batch_size(&mut self, size: u64) {
        unsafe {
            crocksdb_ffi::ctitandb_options_set_min_gc_batch_size(self.inner, size);
        }
    }

    pub fn set_max_gc_batch_size(&mut self, size: u64) {
        unsafe {
            crocksdb_ffi::ctitandb_options_set_max_gc_batch_size(self.inner, size);
        }
    }

    pub fn set_blob_cache(&mut self, cache: &Cache) {
        unsafe {
            crocksdb_ffi::ctitandb_options_set_blob_cache(self.inner, cache.inner);
        }
    }

    pub fn set_discardable_ratio(&mut self, ratio: f64) {
        unsafe {
            crocksdb_ffi::ctitandb_options_set_discardable_ratio(self.inner, ratio);
        }
    }

    pub fn set_merge_small_file_threshold(&mut self, size: u64) {
        unsafe {
            crocksdb_ffi::ctitandb_options_set_merge_small_file_threshold(self.inner, size);
        }
    }

    pub fn set_blob_run_mode(&mut self, t: DBTitanDBBlobRunMode) {
        unsafe {
            crocksdb_ffi::ctitandb_options_set_blob_run_mode(self.inner, t);
        }
    }
}

impl Drop for TitanDBOptions {
    fn drop(&mut self) {
        unsafe {
            crocksdb_ffi::ctitandb_options_destroy(self.inner);
        }
    }
}

#[derive(Debug, Default)]
pub struct TitanBlobIndex {
    inner: DBTitanBlobIndex,
}

impl TitanBlobIndex {
    pub fn decode(value: &[u8]) -> Result<Self, String> {
        let mut index = Self::default();
        unsafe {
            ffi_try!(ctitandb_decode_blob_index(
                value.as_ptr(),
                value.len() as u64,
                &mut index.inner as *mut DBTitanBlobIndex
            ));
        }
        Ok(index)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut value = ptr::null_mut();
        let mut value_size: u64 = 0;
        unsafe {
            ctitandb_encode_blob_index(&self.inner, &mut value, &mut value_size);
            let slice = slice::from_raw_parts(value, value_size as usize);
            let vec = slice.to_vec();
            libc::free(value as *mut libc::c_void);
            vec
        }
    }
}

impl Deref for TitanBlobIndex {
    type Target = DBTitanBlobIndex;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TitanBlobIndex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_index_raw_size() {
        let mut options = TitanDBOptions::new();
        assert!(!options.enable_blob_index_raw_size());
        options.set_enable_blob_index_raw_size(true);
        assert!(options.enable_blob_index_raw_size());
        let legacy = [1, 1, 2, 3];
        let mut index = TitanBlobIndex::decode(&legacy).unwrap();
        assert!(!index.has_raw_value_size);
        assert_eq!(index.encode(), legacy);
        for size in [0, 1, 1_000_000, u64::MAX] {
            index.has_raw_value_size = true;
            index.raw_value_size = size;
            let encoded = index.encode();
            let decoded = TitanBlobIndex::decode(&encoded).unwrap();
            assert!(decoded.has_raw_value_size);
            assert_eq!(decoded.raw_value_size, size);
            assert_eq!(decoded.file_number, 1);
            assert_eq!(decoded.blob_offset, 2);
            assert_eq!(decoded.blob_size, 3);
            assert!(TitanBlobIndex::decode(&encoded[..encoded.len() - 1]).is_err());
        }
        assert!(TitanBlobIndex::decode(&[3, 1, 2, 3]).is_err());
        assert!(TitanBlobIndex::decode(&[1, 1, 2, 3, 0]).is_err());
    }
}
