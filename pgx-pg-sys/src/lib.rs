// Copyright 2020 ZomboDB, LLC <zombodb@gmail.com>. All rights reserved. Use of this source code is
// governed by the MIT license that can be found in the LICENSE file.

//
// we allow improper_ctypes just to eliminate these warnings:
//      = note: `#[warn(improper_ctypes)]` on by default
//      = note: 128-bit integers don't currently have a known stable ABI
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(improper_ctypes)]
#![allow(clippy::unneeded_field_pattern)]

#[cfg(
    any(
        // no features at all will cause problems
        not(any(feature = "pg10", feature = "pg11", feature = "pg12", feature = "pg13")),
  ))]
std::compile_error!("exactly one one feature must be provided (pg10, pg11, or pg12)");

pub mod submodules;
pub use submodules::guard;
pub use submodules::*;

//
// our actual bindings modules -- these are generated by build.rs
//

// feature gate each pg version module
#[cfg(feature = "pg10")]
mod pg10 {
    include!(concat!(env!("OUT_DIR"), "/pg10.rs"));
}
#[cfg(feature = "pg11")]
mod pg11{
    include!(concat!(env!("OUT_DIR"), "/pg11.rs"));
}
#[cfg(feature = "pg12")]
mod pg12 {
    include!(concat!(env!("OUT_DIR"), "/pg12.rs"));
}
#[cfg(feature = "pg13")]
mod pg13 {
    include!(concat!(env!("OUT_DIR"), "/pg13.rs"));
}

// export each module publicly
#[cfg(feature = "pg10")]
pub use pg10::*;
#[cfg(feature = "pg11")]
pub use pg11::*;
#[cfg(feature = "pg12")]
pub use pg12::*;
#[cfg(feature = "pg13")]
pub use pg13::*;

// feature gate each pg-specific oid module
#[cfg(feature = "pg10")]
mod pg10_oids {
    include!(concat!(env!("OUT_DIR"), "/pg10_oids.rs"));
}
#[cfg(feature = "pg11")]
mod pg11_oids {
    include!(concat!(env!("OUT_DIR"), "/pg11_oids.rs"));
}
#[cfg(feature = "pg12")]
mod pg12_oids {
    include!(concat!(env!("OUT_DIR"), "/pg12_oids.rs"));
}
#[cfg(feature = "pg13")]
mod pg13_oids {
    include!(concat!(env!("OUT_DIR"), "/pg13_oids.rs"));
}

// export that module publicly
#[cfg(feature = "pg10")]
pub use pg10_oids::*;
#[cfg(feature = "pg11")]
pub use pg11_oids::*;
#[cfg(feature = "pg12")]
pub use pg12_oids::*;
#[cfg(feature = "pg13")]
pub use pg13_oids::*;

// expose things we want available for all versions
pub use all_versions::*;

// and things that are version-specific
#[cfg(feature = "pg10")]
pub use internal::pg10::add_bool_reloption;
#[cfg(feature = "pg10")]
pub use internal::pg10::add_int_reloption;
#[cfg(feature = "pg10")]
pub use internal::pg10::add_string_reloption;
#[cfg(feature = "pg10")]
pub use internal::pg10::IndexBuildHeapScan;
#[cfg(feature = "pg10")]
pub use internal::pg10::*;

#[cfg(feature = "pg11")]
pub use internal::pg11::IndexBuildHeapScan;
#[cfg(feature = "pg11")]
pub use internal::pg11::*;

#[cfg(feature = "pg12")]
pub use internal::pg12::*;

#[cfg(feature = "pg13")]
pub use internal::pg13::*;

/// A trait applied to all of Postgres' `pg_sys::Node` types and its subtypes
pub trait PgNode {
    type NodeType;

    /// Represent this node as a mutable pointer of its type
    #[inline]
    fn as_node_ptr(&self) -> *mut Self::NodeType {
        self as *const _ as *mut Self::NodeType
    }
}

/// implementation function for `impl Display for $NodeType`
pub(crate) fn node_to_string_for_display(node: *mut crate::Node) -> String {
    unsafe {
        // crate::nodeToString() will never return a null pointer
        let node_to_string = crate::nodeToString(node as *mut std::ffi::c_void);

        let result = match std::ffi::CStr::from_ptr(node_to_string).to_str() {
            Ok(cstr) => cstr.to_string(),
            Err(e) => format!("<ffi error: {:?}>", e),
        };

        crate::pfree(node_to_string as *mut std::ffi::c_void);

        result
    }
}

/// A trait for converting a thing into a `char *` that is allocated by Postgres' palloc
pub trait AsPgCStr {
    fn as_pg_cstr(&self) -> *mut std::os::raw::c_char;
}

impl<'a> AsPgCStr for &'a str {
    fn as_pg_cstr(&self) -> *mut std::os::raw::c_char {
        let self_bytes = self.as_bytes();
        let pg_cstr = unsafe { crate::palloc0(self_bytes.len() + 1) as *mut std::os::raw::c_uchar };
        let slice = unsafe { std::slice::from_raw_parts_mut(pg_cstr, self_bytes.len()) };
        slice.copy_from_slice(self_bytes);
        pg_cstr as *mut std::os::raw::c_char
    }
}

impl AsPgCStr for String {
    fn as_pg_cstr(&self) -> *mut std::os::raw::c_char {
        self.as_str().as_pg_cstr()
    }
}

/// item declarations we want to add to all versions
mod all_versions {
    use crate as pg_sys;
    use pgx_macros::*;

    use memoffset::*;
    use std::str::FromStr;

    /// this comes from `postgres_ext.h`
    pub const InvalidOid: super::Oid = 0;
    pub const InvalidOffsetNumber: super::OffsetNumber = 0;
    pub const FirstOffsetNumber: super::OffsetNumber = 1;
    pub const MaxOffsetNumber: super::OffsetNumber =
        (super::BLCKSZ as usize / std::mem::size_of::<super::ItemIdData>()) as super::OffsetNumber;
    pub const InvalidBlockNumber: u32 = 0xFFFF_FFFF as crate::BlockNumber;
    pub const VARHDRSZ: usize = std::mem::size_of::<super::int32>();
    pub const InvalidTransactionId: super::TransactionId = 0 as super::TransactionId;
    pub const InvalidCommandId: super::CommandId = (!(0 as super::CommandId)) as super::CommandId;
    pub const FirstCommandId: super::CommandId = 0 as super::CommandId;
    pub const BootstrapTransactionId: super::TransactionId = 1 as super::TransactionId;
    pub const FrozenTransactionId: super::TransactionId = 2 as super::TransactionId;
    pub const FirstNormalTransactionId: super::TransactionId = 3 as super::TransactionId;
    pub const MaxTransactionId: super::TransactionId = 0xFFFF_FFFF as super::TransactionId;

    #[pgx_macros::pg_guard]
    extern "C" {
        pub fn pgx_list_nth(list: *mut super::List, nth: i32) -> *mut std::os::raw::c_void;
        pub fn pgx_list_nth_int(list: *mut super::List, nth: i32) -> i32;
        pub fn pgx_list_nth_oid(list: *mut super::List, nth: i32) -> super::Oid;
        pub fn pgx_list_nth_cell(list: *mut super::List, nth: i32) -> *mut super::ListCell;
        pub fn pgx_GETSTRUCT(tuple: pg_sys::HeapTuple) -> *mut std::os::raw::c_char;
    }

    #[inline]
    pub fn VARHDRSZ_EXTERNAL() -> usize {
        offset_of!(super::varattrib_1b_e, va_data)
    }

    #[inline]
    pub fn VARHDRSZ_SHORT() -> usize {
        offset_of!(super::varattrib_1b, va_data)
    }

    #[inline]
    pub fn get_pg_major_version_string() -> &'static str {
        let mver = std::ffi::CStr::from_bytes_with_nul(super::PG_MAJORVERSION).unwrap();
        mver.to_str().unwrap()
    }

    #[inline]
    pub fn get_pg_major_version_num() -> u16 {
        u16::from_str(super::get_pg_major_version_string()).unwrap()
    }

    #[inline]
    pub fn get_pg_version_string() -> &'static str {
        let ver = std::ffi::CStr::from_bytes_with_nul(super::PG_VERSION_STR).unwrap();
        ver.to_str().unwrap()
    }

    #[inline]
    pub fn get_pg_major_minor_version_string() -> &'static str {
        let mver = std::ffi::CStr::from_bytes_with_nul(super::PG_VERSION).unwrap();
        mver.to_str().unwrap()
    }

    #[inline]
    pub fn TransactionIdIsNormal(xid: super::TransactionId) -> bool {
        xid >= FirstNormalTransactionId
    }

    /// ```c
    ///     #define type_is_array(typid)  (get_element_type(typid) != InvalidOid)
    /// ```
    #[inline]
    pub unsafe fn type_is_array(typoid: super::Oid) -> bool {
        super::get_element_type(typoid) != InvalidOid
    }

    #[inline]
    pub unsafe fn planner_rt_fetch(
        index: super::Index,
        root: *mut super::PlannerInfo,
    ) -> *mut super::RangeTblEntry {
        extern "C" {
            pub fn pgx_planner_rt_fetch(
                index: super::Index,
                root: *mut super::PlannerInfo,
            ) -> *mut super::RangeTblEntry;
        }

        pgx_planner_rt_fetch(index, root)
    }

    /// ```c
    /// #define rt_fetch(rangetable_index, rangetable) \
    ///     ((RangeTblEntry *) list_nth(rangetable, (rangetable_index)-1))
    /// ```
    #[inline]
    pub unsafe fn rt_fetch(
        index: super::Index,
        range_table: *mut super::List,
    ) -> *mut super::RangeTblEntry {
        pgx_list_nth(range_table, index as i32 - 1) as *mut super::RangeTblEntry
    }

    #[inline]
    pub fn HeapTupleHeaderGetXmin(
        htup_header: super::HeapTupleHeader,
    ) -> Option<super::TransactionId> {
        extern "C" {
            pub fn pgx_HeapTupleHeaderGetXmin(
                htup_header: super::HeapTupleHeader,
            ) -> super::TransactionId;
        }

        if htup_header.is_null() {
            None
        } else {
            Some(unsafe { pgx_HeapTupleHeaderGetXmin(htup_header) })
        }
    }

    #[inline]
    pub fn HeapTupleHeaderGetRawCommandId(
        htup_header: super::HeapTupleHeader,
    ) -> Option<super::CommandId> {
        extern "C" {
            pub fn pgx_HeapTupleHeaderGetRawCommandId(
                htup_header: super::HeapTupleHeader,
            ) -> super::CommandId;
        }

        if htup_header.is_null() {
            None
        } else {
            Some(unsafe { pgx_HeapTupleHeaderGetRawCommandId(htup_header) })
        }
    }

    /// #define HeapTupleHeaderIsHeapOnly(tup) \
    ///    ( \
    ///       ((tup)->t_infomask2 & HEAP_ONLY_TUPLE) != 0 \
    ///    )
    #[inline]
    pub unsafe fn HeapTupleHeaderIsHeapOnly(htup_header: super::HeapTupleHeader) -> bool {
        ((*htup_header).t_infomask2 & crate::HEAP_ONLY_TUPLE as u16) != 0
    }

    /// #define HeapTupleHeaderIsHotUpdated(tup) \
    /// ( \
    ///      ((tup)->t_infomask2 & HEAP_HOT_UPDATED) != 0 && \
    ///      ((tup)->t_infomask & HEAP_XMAX_INVALID) == 0 && \
    ///      !HeapTupleHeaderXminInvalid(tup) \
    /// )
    #[inline]
    pub unsafe fn HeapTupleHeaderIsHotUpdated(htup_header: super::HeapTupleHeader) -> bool {
        (*htup_header).t_infomask2 & crate::HEAP_HOT_UPDATED as u16 != 0
            && (*htup_header).t_infomask & crate::HEAP_XMAX_INVALID as u16 == 0
            && !HeapTupleHeaderXminInvalid(htup_header)
    }

    /// #define HeapTupleHeaderXminInvalid(tup) \
    /// ( \
    ///   ((tup)->t_infomask & (HEAP_XMIN_COMMITTED|HEAP_XMIN_INVALID)) == \
    ///      HEAP_XMIN_INVALID \
    /// )
    #[inline]
    pub unsafe fn HeapTupleHeaderXminInvalid(htup_header: super::HeapTupleHeader) -> bool {
        (*htup_header).t_infomask
            & (crate::HEAP_XMIN_COMMITTED as u16 | crate::HEAP_XMIN_INVALID as u16)
            == crate::HEAP_XMIN_INVALID as u16
    }

    /// #define BufferGetPage(buffer) ((Page)BufferGetBlock(buffer))
    #[inline]
    pub unsafe fn BufferGetPage(buffer: crate::Buffer) -> crate::Page {
        BufferGetBlock(buffer) as crate::Page
    }

    /// #define BufferGetBlock(buffer) \
    /// ( \
    ///      AssertMacro(BufferIsValid(buffer)), \
    ///      BufferIsLocal(buffer) ? \
    ///            LocalBufferBlockPointers[-(buffer) - 1] \
    ///      : \
    ///            (Block) (BufferBlocks + ((Size) ((buffer) - 1)) * BLCKSZ) \
    /// )
    #[inline]
    pub unsafe fn BufferGetBlock(buffer: crate::Buffer) -> crate::Block {
        if BufferIsLocal(buffer) {
            *crate::LocalBufferBlockPointers.offset(((-buffer) - 1) as isize)
        } else {
            crate::BufferBlocks
                .offset((((buffer as crate::Size) - 1) * crate::BLCKSZ as usize) as isize)
                as crate::Block
        }
    }

    /// #define BufferIsLocal(buffer)      ((buffer) < 0)
    #[inline]
    pub unsafe fn BufferIsLocal(buffer: crate::Buffer) -> bool {
        buffer < 0
    }

    #[inline]
    pub fn heap_tuple_get_struct<T>(htup: super::HeapTuple) -> *mut T {
        if htup.is_null() {
            0 as *mut T
        } else {
            unsafe { pgx_GETSTRUCT(htup) as *mut T }
        }
    }

    #[pg_guard]
    extern "C" {
        pub fn query_tree_walker(
            query: *mut super::Query,
            walker: ::std::option::Option<
                unsafe extern "C" fn(*mut super::Node, *mut ::std::os::raw::c_void) -> bool,
            >,
            context: *mut ::std::os::raw::c_void,
            flags: ::std::os::raw::c_int,
        ) -> bool;
    }

    #[pg_guard]
    extern "C" {
        pub fn expression_tree_walker(
            node: *mut super::Node,
            walker: ::std::option::Option<
                unsafe extern "C" fn(*mut super::Node, *mut ::std::os::raw::c_void) -> bool,
            >,
            context: *mut ::std::os::raw::c_void,
        ) -> bool;
    }
}

mod internal {
    //
    // for specific versions
    //

    #[cfg(feature = "pg10")]
    pub(crate) mod pg10 {
        use crate::pg10::*;

        pub use crate::pg10::tupleDesc as TupleDescData;
        pub use crate::pg10::AllocSetContextCreate as AllocSetContextCreateExtended;
        pub type QueryCompletion = std::os::raw::c_char;

        pub unsafe fn add_string_reloption(
            kinds: bits32,
            name: *const ::std::os::raw::c_char,
            desc: *const ::std::os::raw::c_char,
            default_val: *const ::std::os::raw::c_char,
            validator: ::std::option::Option<
                unsafe extern "C" fn(value: *const ::std::os::raw::c_char),
            >,
        ) {
            // PG10 defines the validator function as taking a "*mut c_char"
            // whereas PG11/12 want a "*const c_char".
            //
            // For ease of use by users of this crate, we cast the provided
            // 'validator' function to what PG10 wants, using transmute
            //
            // If there's a better way to do this, I'ld love to know!
            let func_as_mut_arg = match validator {
                Some(func) => {
                    let func_ptr = std::mem::transmute::<
                        unsafe extern "C" fn(*const ::std::os::raw::c_char),
                        unsafe extern "C" fn(*mut ::std::os::raw::c_char),
                    >(func);
                    Some(func_ptr)
                }
                None => None,
            };

            crate::pg10::add_string_reloption(
                kinds,
                name as *mut std::os::raw::c_char,
                desc as *mut std::os::raw::c_char,
                default_val as *mut std::os::raw::c_char,
                func_as_mut_arg,
            );
        }

        pub unsafe fn add_int_reloption(
            kinds: bits32,
            name: *const ::std::os::raw::c_char,
            desc: *const ::std::os::raw::c_char,
            default_val: ::std::os::raw::c_int,
            min_val: ::std::os::raw::c_int,
            max_val: ::std::os::raw::c_int,
        ) {
            crate::pg10::add_int_reloption(
                kinds,
                name as *mut std::os::raw::c_char,
                desc as *mut std::os::raw::c_char,
                default_val,
                min_val,
                max_val,
            );
        }

        pub unsafe fn add_bool_reloption(
            kinds: bits32,
            name: *const ::std::os::raw::c_char,
            desc: *const ::std::os::raw::c_char,
            default_val: bool,
        ) {
            crate::pg10::add_bool_reloption(
                kinds,
                name as *mut std::os::raw::c_char,
                desc as *mut std::os::raw::c_char,
                default_val,
            );
        }

        /// # Safety
        ///
        /// This function wraps Postgres' internal `IndexBuildHeapScan` method, and therefore, is
        /// inherently unsafe
        pub unsafe fn IndexBuildHeapScan<T>(
            heap_relation: crate::Relation,
            index_relation: crate::Relation,
            index_info: *mut crate::pg10::IndexInfo,
            build_callback: crate::IndexBuildCallback,
            build_callback_state: *mut T,
        ) {
            crate::pg10::IndexBuildHeapScan(
                heap_relation,
                index_relation,
                index_info,
                true,
                build_callback,
                build_callback_state as *mut std::os::raw::c_void,
            );
        }
    }

    #[cfg(feature = "pg11")]
    pub(crate) mod pg11 {
        pub use crate::pg11::tupleDesc as TupleDescData;
        pub type QueryCompletion = std::os::raw::c_char;

        /// # Safety
        ///
        /// This function wraps Postgres' internal `IndexBuildHeapScan` method, and therefore, is
        /// inherently unsafe
        pub unsafe fn IndexBuildHeapScan<T>(
            heap_relation: crate::Relation,
            index_relation: crate::Relation,
            index_info: *mut crate::pg11::IndexInfo,
            build_callback: crate::IndexBuildCallback,
            build_callback_state: *mut T,
        ) {
            crate::pg11::IndexBuildHeapScan(
                heap_relation,
                index_relation,
                index_info,
                true,
                build_callback,
                build_callback_state as *mut std::os::raw::c_void,
                std::ptr::null_mut(),
            );
        }
    }

    #[cfg(feature = "pg12")]
    pub(crate) mod pg12 {
        pub use crate::pg12::AllocSetContextCreateInternal as AllocSetContextCreateExtended;
        pub type QueryCompletion = std::os::raw::c_char;

        pub const QTW_EXAMINE_RTES: u32 = crate::pg12::QTW_EXAMINE_RTES_BEFORE;

        /// # Safety
        ///
        /// This function wraps Postgres' internal `IndexBuildHeapScan` method, and therefore, is
        /// inherently unsafe
        pub unsafe fn IndexBuildHeapScan<T>(
            heap_relation: crate::Relation,
            index_relation: crate::Relation,
            index_info: *mut crate::pg12::IndexInfo,
            build_callback: crate::IndexBuildCallback,
            build_callback_state: *mut T,
        ) {
            let heap_relation_ref = heap_relation.as_ref().unwrap();
            let table_am = heap_relation_ref.rd_tableam.as_ref().unwrap();

            table_am.index_build_range_scan.unwrap()(
                heap_relation,
                index_relation,
                index_info,
                true,
                false,
                true,
                0,
                crate::InvalidBlockNumber,
                build_callback,
                build_callback_state as *mut std::os::raw::c_void,
                std::ptr::null_mut(),
            );
        }
    }

    #[cfg(feature = "pg13")]
    pub(crate) mod pg13 {
        pub use crate::pg13::AllocSetContextCreateInternal as AllocSetContextCreateExtended;

        pub const QTW_EXAMINE_RTES: u32 = crate::pg13::QTW_EXAMINE_RTES_BEFORE;

        /// # Safety
        ///
        /// This function wraps Postgres' internal `IndexBuildHeapScan` method, and therefore, is
        /// inherently unsafe
        pub unsafe fn IndexBuildHeapScan<T>(
            heap_relation: crate::Relation,
            index_relation: crate::Relation,
            index_info: *mut crate::IndexInfo,
            build_callback: crate::IndexBuildCallback,
            build_callback_state: *mut T,
        ) {
            let heap_relation_ref = heap_relation.as_ref().unwrap();
            let table_am = heap_relation_ref.rd_tableam.as_ref().unwrap();

            table_am.index_build_range_scan.unwrap()(
                heap_relation,
                index_relation,
                index_info,
                true,
                false,
                true,
                0,
                crate::InvalidBlockNumber,
                build_callback,
                build_callback_state as *mut std::os::raw::c_void,
                std::ptr::null_mut(),
            );
        }
    }
}
