use libc::{c_void, c_char, c_int, c_uint};
use common::*;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct JsModuleRecord(pub *mut c_void);

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum JsParseModuleSourceFlags {
    DataIsUTF16LE = 0,
    DataIsUTF8 = 1,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum JsModuleHostInfoKind {
    Exception = 1,
    HostDefined = 2,
    NotifyModuleReadyCallback = 3,
    FetchImportedModuleCallback = 4,
}

pub type FetchImportedModuleCallBack =
    Option<extern "system" fn(referencingModule: JsModuleRecord,
                              specifier: JsValueRef,
                              dependentModuleRecord: *mut JsModuleRecord)
                              -> JsErrorCode>;

pub type NotifyModuleReadyCallback = Option<extern "system" fn(referencingModule: JsModuleRecord,
                                                               exceptionVar: JsValueRef)
                                                               -> JsErrorCode>;

pub type JsSerializedLoadScriptCallback =
    Option<extern "system" fn(sourceContext: JsSourceContext,
                              value: *mut JsValueRef,
                              parseAttributes: *mut JsParseScriptAttributes)
                              -> bool>;

#[link(name = "ChakraCore")]
extern "system" {
    pub fn JsInitializeModuleRecord(referencingModule: JsModuleRecord,
                                    normalizedSpecifier: JsValueRef,
                                    moduleRecord: *mut JsModuleRecord)
                                    -> JsErrorCode;
    pub fn JsParseModuleSource(requestModule: JsModuleRecord,
                               sourceContext: JsSourceContext,
                               script: *mut u8,
                               scriptLength: c_uint,
                               sourceFlag: JsParseModuleSourceFlags,
                               exceptionValueRef: *mut JsValueRef)
                               -> JsErrorCode;
    pub fn JsModuleEvaluation(requestModule: JsModuleRecord,
                              result: *mut JsValueRef)
                              -> JsErrorCode;
    pub fn JsSetModuleHostInfo(requestModule: JsModuleRecord,
                               moduleHostInfo: JsModuleHostInfoKind,
                               hostInfo: *mut c_void)
                               -> JsErrorCode;
    pub fn JsGetModuleHostInfo(requestModule: JsModuleRecord,
                               moduleHostInfo: JsModuleHostInfoKind,
                               hostInfo: *mut *mut c_void)
                               -> JsErrorCode;
    pub fn JsCreateString(content: *const c_char,
                          length: usize,
                          value: *mut JsValueRef)
                          -> JsErrorCode;
    pub fn JsCreateStringUtf8(content: *const u8,
                              length: usize,
                              value: *mut JsValueRef)
                              -> JsErrorCode;
    pub fn JsCreateStringUtf16(content: *const u16,
                               length: usize,
                               value: *mut JsValueRef)
                               -> JsErrorCode;
    pub fn JsCopyString(value: JsValueRef,
                        start: c_int,
                        length: c_int,
                        buffer: *mut c_char,
                        written: *mut usize)
                        -> JsErrorCode;
    pub fn JsCopyStringUtf8(value: JsValueRef,
                            buffer: *mut u8,
                            bufferSize: usize,
                            written: *mut usize)
                            -> JsErrorCode;
    pub fn JsCopyStringUtf16(value: JsValueRef,
                             start: c_int,
                             length: c_int,
                             buffer: *mut u16,
                             written: *mut usize)
                             -> JsErrorCode;
    pub fn JsParse(script: JsValueRef,
                   sourceContext: JsSourceContext,
                   sourceUrl: JsValueRef,
                   parseAttributes: JsParseScriptAttributes,
                   result: *mut JsValueRef)
                   -> JsErrorCode;
    pub fn JsRun(script: JsValueRef,
                 sourceContext: JsSourceContext,
                 sourceUrl: JsValueRef,
                 parseAttributes: JsParseScriptAttributes,
                 result: *mut JsValueRef)
                 -> JsErrorCode;
    pub fn JsCreatePropertyIdUtf8(name: *const c_char,
                                  length: usize,
                                  propertyId: *mut JsPropertyIdRef)
                                  -> JsErrorCode;
    pub fn JsCopyPropertyIdUtf8(propertyId: JsPropertyIdRef,
                                buffer: *mut u8,
                                bufferSize: usize,
                                length: *mut usize)
                                -> JsErrorCode;
    pub fn JsSerialize(script: JsValueRef,
                       buffer: *mut u8,
                       bufferSize: *mut c_uint,
                       parseAttributes: JsParseScriptAttributes)
                       -> JsErrorCode;
    pub fn JsParseSerialized(buffer: *mut u8,
                             scriptLoadCallback: JsSerializedLoadScriptCallback,
                             sourceContext: JsSourceContext,
                             sourceUrl: JsValueRef,
                             result: *mut JsValueRef)
                             -> JsErrorCode;
    pub fn JsRunSerialized(buffer: *mut u8,
                           scriptLoadCallback: JsSerializedLoadScriptCallback,
                           sourceContext: JsSourceContext,
                           sourceUrl: JsValueRef,
                           result: *mut JsValueRef)
                           -> JsErrorCode;
}
