use std::path::Path;

use cxx::{let_cxx_string, UniquePtr};
use thiserror::Error;

pub struct Mesh {
    inner: UniquePtr<mfem_sys::ffi::Mesh>,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            inner: mfem_sys::ffi::Mesh_ctor(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, MfemError> {
        let generate_edges = 1;
        let refine = 1;
        let fix_orientation = true;
        let_cxx_string!(mesh_path = path);
        Ok(Self {
            inner: mfem_sys::ffi::Mesh_ctor_file(
                &mesh_path,
                generate_edges,
                refine,
                fix_orientation,
            ),
        })
    }

    pub fn dimension(&self) -> i32 {
        self.inner.Dimension()
    }

    pub fn get_num_elems(&self) -> i32 {
        self.inner.GetNE()
    }
}

#[derive(Error, Debug)]
pub enum MfemError {}
