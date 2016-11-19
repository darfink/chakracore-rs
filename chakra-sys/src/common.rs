use std::ptr;
use libc::{c_void, c_ushort, c_int, c_uint};

/// An error code returned from a Chakra hosting API.
#[repr(C)]
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum JsErrorCode {
    /// Success error code.
    NoError = 0,
    /// Category of errors that relates to incorrect usage of the API itself.
    CategoryUsage = 0x10000,
    /// An argument to a hosting API was invalid.
    InvalidArgument,
    /// An argument to a hosting API was null in a context where null is not allowed.
    NullArgument,
    /// The hosting API requires that a context be current, but there is no
    /// current context.
    NoCurrentContext,
    /// The engine is in an exception state and no APIs can be called until
    /// the exception is cleared.
    InExceptionState,
    /// A hosting API is not yet implemented.
    NotImplemented,
    /// A hosting API was called on the wrong thread.
    WrongThread,
    /// A runtime that is still in use cannot be disposed.
    RuntimeInUse,
    /// A bad serialized script was used, or the serialized script was
    /// serialized by a different version of the Chakra engine.
    BadSerializedScript,
    /// The runtime is in a disabled state.
    InDisabledState,
    /// Runtime does not support reliable script interruption.
    CannotDisableExecution,
    /// A heap enumeration is currently underway in the script context.
    HeapEnumInProgress,
    /// A hosting API that operates on object values was called with a non-object value.
    ArgumentNotObject,
    /// A script context is in the middle of a profile callback.
    InProfileCallback,
    /// A thread service callback is currently underway.
    InThreadServiceCallback,
    /// Scripts cannot be serialized in debug contexts.
    CannotSerializeDebugScript,
    /// The context cannot be put into a debug state because it is already in a debug state.
    AlreadyDebuggingContext,
    /// The context cannot start profiling because it is already profiling.
    AlreadyProfilingContext,
    /// Idle notification given when the host did not enable idle processing.
    IdleNotEnabled,
    /// The context did not accept the enqueue callback.
    CannotSetProjectionEnqueueCallback,
    /// Failed to start projection.
    CannotStartProjection,
    /// The operation is not supported in an object before collect callback.
    InObjectBeforeCollectCallback,
    /// Object cannot be unwrapped to IInspectable pointer.
    ObjectNotInspectable,
    /// A hosting API that operates on symbol property ids but was called
    /// with a non-symbol property id. The error code is returned by
    /// JsGetSymbolFromPropertyId if the function is called with non-symbol
    /// property id.
    PropertyNotSymbol,
    /// A hosting API that operates on string property ids but was called
    /// with a non-string property id. The error code is returned by
    /// existing JsGetPropertyNamefromId if the function is called with
    /// non-string property id.
    PropertyNotString,
    /// Module evaulation is called in wrong context.
    InvalidContext,
    /// Module evaulation is called in wrong context.
    InvalidModuleHostInfoKind,
    /// Module was parsed already when JsParseModuleSource is called.
    ModuleParsed,
    /// Module was evaluated already when JsModuleEvaluation is called.
    ModuleEvaluated,
    /// Category of errors that relates to errors occurring within the engine itself.
    CategoryEngine = 0x20000,
    /// The Chakra engine has run out of memory.
    OutOfMemory,
    /// The Chakra engine failed to set the Floating Point Unit state.
    BadFPUState,
    /// Category of errors that relates to errors in a script.
    CategoryScript = 0x30000,
    /// A JavaScript exception occurred while running a script.
    ScriptException,
    /// JavaScript failed to compile.
    ScriptCompile,
    /// A script was terminated due to a request to suspend a runtime.
    ScriptTerminated,
    /// A script was terminated because it tried to use `eval or
    /// `function` and eval was disabled.
    ScriptEvalDisabled,
    /// Category of errors that are fatal and signify failure of the engine.
    CategoryFatal = 0x40000,
    /// A fatal error in the engine has occurred.
    Fatal,
    /// A hosting API was called with object created on different javascript runtime.
    WrongRuntime,
    /// Category of errors that are related to failures during diagnostic operations.
    CategoryDiagError = 0x50000,
    /// The object for which the debugging API was called was not found
    DiagAlreadyInDebugMode,
    /// The debugging API can only be called when VM is in debug mode
    DiagNotInDebugMode,
    /// The debugging API can only be called when VM is at a break
    DiagNotAtBreak,
    /// Debugging API was called with an invalid handle.
    DiagInvalidHandle,
    /// The object for which the debugging API was called was not found
    DiagObjectNotFound,
    /// VM was unable to perfom the request action
    DiagUnableToPerformAction,
}

/// A handle to a Chakra runtime.
///
/// Each Chakra runtime has its own independent execution engine, JIT compiler,
/// and garbage collected heap. As such, each runtime is completely isolated
/// from other runtimes.
///
/// Runtimes can be used on any thread, but only one thread can call into a
/// runtime at any time.
///
/// NOTE: A `JsRuntimeHandle`, unlike other object references in the Chakra
/// hosting API, is not garbage collected since it contains the garbage
/// collected heap itself. A runtime will continue to exist until
/// `JsDisposeRuntime` is called.
#[repr(C)]
#[derive(PartialEq, Copy, Clone, Debug)]
pub struct JsRuntimeHandle(pub *mut c_void);

impl JsRuntimeHandle {
    pub fn new() -> Self {
        JsRuntimeHandle(ptr::null_mut())
    }
}

/// A reference to an object owned by the Chakra garbage collector.
///
/// A Chakra runtime will automatically track `JsRef` references as long as they
/// are stored in local variables or in parameters (i.e. on the stack). Storing
/// a `JsRef` somewhere other than on the stack requires calling `JsAddRef` and
/// `JsRelease` to manage the lifetime of the object, otherwise the garbage
/// collector may free the object while it is still in use.
#[repr(C)]
#[derive(PartialEq, Copy, Clone, Debug)]
pub struct JsRef(pub *mut c_void);

impl JsRef {
    pub fn new() -> Self {
        JsRef(ptr::null_mut())
    }
}

/// A reference to a script context.
///
/// Each script context contains its own global object, distinct from the global
/// object in other script contexts.
///
/// Many Chakra hosting APIs require an "active" script context, which can be
/// set using `JsSetCurrentContext`. Chakra hosting APIs that require a current
/// context to be set will note that explicitly in their documentation.
pub type JsContextRef = JsRef;

/// A reference to a JavaScript value.
///
/// A JavaScript value is one of the following types of values: undefined, null,
/// Boolean, string, number, or object.
pub type JsValueRef = JsRef;

/// A cookie that identifies a script for debugging purposes.
pub type JsSourceContext = *mut usize;

/// An empty source context.
// pub const JS_SOURCE_CONTEXT_NONE: JsSourceContext = usize::max_value() as *mut _;
/// A property identifier.
///
/// Property identifiers are used to refer to properties of JavaScript objects
/// instead of using strings.
pub type JsPropertyIdRef = JsRef;

bitflags! {
    /// Attributes of a runtime.
    pub flags JsRuntimeAttributes: c_int {
        /// No special attributes.
        const JsRuntimeAttributeNone = 0,
        /// The runtime will not do any work (such as garbage collection) on
        /// background threads.
        const JsRuntimeAttributeDisableBackgroundWork = (1 << 0),
        /// The runtime should support reliable script interruption. This increases
        /// the number of places where the runtime will check for a script interrupt
        /// request at the cost of a small amount of runtime performance.
        const JsRuntimeAttributeAllowScriptInterrupt = (1 << 1),
        /// Host will call `JsIdle`, so enable idle processing. Otherwise, the
        /// runtime will manage memory slightly more aggressively.
        const JsRuntimeAttributeEnableIdleProcessing = (1 << 2),
        /// Runtime will not generate native code.
        const JsRuntimeAttributeDisableNativeCodeGeneration = (1 << 3),
        /// Using `eval` or `function` constructor will throw an exception.
        const JsRuntimeAttributeDisableEval = (1 << 4),
        /// Runtime will enable all experimental features.
        const JsRuntimeAttributeEnableExperimentalFeatures = (1 << 5),
        /// Calling `JsSetException` will also dispatch the exception to the script
        /// debugger (if any) giving the debugger a chance to break on the
        /// exception.
        const JsRuntimeAttributeDispatchSetExceptionsToDebugger = (1 << 6),
    }
}

/// The type of a typed JavaScript array.
#[repr(C)]
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum JsTypedArrayType {
    /// An int8 array.
    Int8,
    /// An uint8 array.
    Uint8,
    /// An uint8 clamped array.
    Uint8Clamped,
    /// An int16 array.
    Int16,
    /// An uint16 array.
    Uint16,
    /// An int32 array.
    Int32,
    /// An uint32 array.
    Uint32,
    /// A float32 array.
    Float32,
    /// A float64 array.
    Float64,
}

/// Allocation callback event type.
#[repr(C)]
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum JsMemoryEventType {
    /// Indicates a request for memory allocation.
    Allocate = 0,
    /// Indicates a memory freeing event.
    Free = 1,
    /// Indicates a failed allocation event.
    Failure = 2,
}

bitflags! {
    /// Attribute mask for JsParseScriptWithAttributes
    pub flags JsParseScriptAttributes: c_int {
        /// Default attribute
        const JsParseScriptAttributeNone = 0,
        /// Specified script is internal and non-user code. Hidden from debugger
        const JsParseScriptAttributeLibraryCode = (1 << 0),
        /// ChakraCore assumes ExternalArrayBuffer is Utf8 by default. This one
        /// needs to be set for Utf16
        const JsParseScriptAttributeArrayBufferIsUtf16Encoded = (1 << 1),
    }
}

/// Type enumeration of a JavaScript property
#[repr(C)]
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum JsPropertyIdType {
    /// Type enumeration of a JavaScript string property
    String,
    /// Type enumeration of a JavaScript symbol property
    Symbol,
}

/// The JavaScript type of a JsValueRef.
#[repr(C)]
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum JsValueType {
    /// The value is the `undefined` value.
    Undefined = 0,
    /// The value is the `null` value.
    Null = 1,
    /// The value is a JavaScript number value.
    Number = 2,
    /// The value is a JavaScript string value.
    String = 3,
    /// The value is a JavaScript Boolean value.
    Boolean = 4,
    /// The value is a JavaScript object value.
    Object = 5,
    /// The value is a JavaScript function object value.
    Function = 6,
    /// The value is a JavaScript error object value.
    Error = 7,
    /// The value is a JavaScript array object value.
    Array = 8,
    /// The value is a JavaScript symbol value.
    Symbol = 9,
    /// The value is a JavaScript ArrayBuffer object value.
    ArrayBuffer = 10,
    /// The value is a JavaScript typed array object value.
    TypedArray = 11,
    /// The value is a JavaScript DataView object value.
    DataView = 12,
}

/// User implemented callback routine for memory allocation events.
///
/// Use `JsSetRuntimeMemoryAllocationCallback` to register this callback.
///
/// - `callbackState`: The state passed to `JsSetRuntimeMemoryAllocationCallback`.
/// - `allocationEvent`: The type of type allocation event.
/// - `allocationSize`: The size of the allocation.
///
/// For the `JsMemoryAllocate` event, returning `true` allows the runtime to
/// continue with the allocation. Returning false indicates the allocation
/// request is rejected. The return value is ignored for other allocation
/// events.
pub type JsMemoryAllocationCallback =
    Option<extern "system" fn(callbackState: *mut c_void,
                              allocationEvent: JsMemoryEventType,
                              allocationSize: usize)
                              -> bool>;

/// A callback called before collection.
///
/// Use `JsSetBeforeCollectCallback` to register this callback.
///
/// - `callbackState` The state passed to `JsSetBeforeCollectCallback`.
pub type JsBeforeCollectCallback = Option<extern "system" fn(callbackState: *mut c_void)>;

/// A callback called before collecting an object.
///
/// Use `JsSetObjectBeforeCollectCallback` to register this callback.
///
/// - `ref`: The object to be collected.
/// - `callbackState`: The state passed to `JsSetObjectBeforeCollectCallback`.
pub type JsObjectBeforeCollectCallback = Option<extern "system" fn(reference: JsRef,
                                                                   callbackState: *mut c_void)>;

/// A background work item callback.
///
/// This is passed to the host's thread service (if provided) to allow the host
/// to invoke the work item callback on the background thread of its choice.
///
/// - `callbackState`: Data argument passed to the thread service.
pub type JsBackgroundWorkItemCallback = Option<extern "system" fn(callbackState: *mut c_void)>;

/// A thread service callback.
///
/// The host can specify a background thread service when calling
/// `JsCreateRuntime`. If specified, then background work items will be passed
/// to the host using this callback. The host is expected to either begin
/// executing the background work item immediately and return true or return
/// false and the runtime will handle the work item in-thread.
///
/// - `callback`: The callback for the background work item.
/// - `callbackState`: The data argument to be passed to the callback.
pub type JsThreadServiceCallback =
    Option<extern "system" fn(callback: JsBackgroundWorkItemCallback,
                              callbackState: *mut c_void)
                              -> bool>;

/// Called by the runtime when it is finished with all resources related to the script execution.
/// The caller should free the source if loaded, the byte code, and the context at this time.
///
/// - `sourceContext`: The context passed to Js[Parse|Run]SerializedScriptWithCallback
pub type JsSerializedScriptUnloadCallback =
    Option<extern "system" fn(sourceContext: JsSourceContext)>;

/// A finalizer callback.
///
/// - `data`: The external data that was passed in when creating the object being finalized.
pub type JsFinalizeCallback = Option<extern "system" fn(data: *mut c_void)>;

/// A function callback.
///
/// - `callee`: A function object that represents the function being invoked.
/// - `isConstructCall`: Indicates whether this is a regular call or a 'new' call.
/// - `arguments`: The arguments to the call.
/// - `argumentCount`: The number of arguments.
/// - `callbackState`: The state passed to `JsCreateFunction`.
///
/// *Returns:* The result of the call, if any.
pub type JsNativeFunction = Option<extern "system" fn(callee: JsValueRef,
                                                      isConstructCall: bool,
                                                      arguments: *mut JsValueRef,
                                                      argumentCount: c_ushort,
                                                      callbackState: *mut c_void)
                                                      -> JsValueRef>;

/// A promise continuation callback.
///
/// The host can specify a promise continuation callback in
/// `JsSetPromiseContinuationCallback`. If a script creates a task to be run
/// later, then the promise continuation callback will be called with the task
/// and the task should be put in a FIFO queue, to be run when the current
/// script is done executing.
///
/// - `task`: The task, represented as a JavaScript function.
/// - `callbackState`: The data argument to be passed to the callback.
pub type JsPromiseContinuationCallback = Option<extern "system" fn(task: JsValueRef,
                                                                   callbackState: *mut c_void)>;

extern "system" {
    pub fn JsCreateRuntime(attributes: JsRuntimeAttributes,
                           threadService: JsThreadServiceCallback,
                           runtime: *mut JsRuntimeHandle)
                           -> JsErrorCode;
    pub fn JsCollectGarbage(runtime: JsRuntimeHandle) -> JsErrorCode;
    pub fn JsDisposeRuntime(runtime: JsRuntimeHandle) -> JsErrorCode;
    pub fn JsGetRuntimeMemoryUsage(runtime: JsRuntimeHandle,
                                   memoryUsage: *mut usize)
                                   -> JsErrorCode;
    pub fn JsGetRuntimeMemoryLimit(runtime: JsRuntimeHandle,
                                   memoryLimit: *mut usize)
                                   -> JsErrorCode;
    pub fn JsSetRuntimeMemoryLimit(runtime: JsRuntimeHandle, memoryLimit: usize) -> JsErrorCode;
    pub fn JsSetRuntimeMemoryAllocationCallback(runtime: JsRuntimeHandle,
                                                callbackState: *mut c_void,
                                                allocationCallback: JsMemoryAllocationCallback)
                                                -> JsErrorCode;
    pub fn JsSetRuntimeBeforeCollectCallback(runtime: JsRuntimeHandle,
                                             callbackState: *mut c_void,
                                             beforeCollectCallback: JsBeforeCollectCallback)
                                             -> JsErrorCode;
    pub fn JsAddRef(reference: JsRef, count: *mut c_uint) -> JsErrorCode;
    pub fn JsRelease(reference: JsRef, count: *mut c_uint) -> JsErrorCode;
    pub fn JsSetObjectBeforeCollectCallback(
        reference: JsRef,
        callbackState: *mut c_void,
        objectBeforeCollectCallback: JsObjectBeforeCollectCallback) -> JsErrorCode;
    pub fn JsCreateContext(runtime: JsRuntimeHandle, newContext: *mut JsContextRef) -> JsErrorCode;
    pub fn JsGetCurrentContext(currentContext: *mut JsContextRef) -> JsErrorCode;
    pub fn JsSetCurrentContext(context: JsContextRef) -> JsErrorCode;
    pub fn JsGetContextOfObject(object: JsValueRef, context: *mut JsContextRef) -> JsErrorCode;
    pub fn JsGetContextData(context: JsContextRef, data: *mut *mut c_void) -> JsErrorCode;
    pub fn JsSetContextData(context: JsContextRef, data: *mut c_void) -> JsErrorCode;
    pub fn JsGetRuntime(context: JsContextRef, runtime: *mut JsRuntimeHandle) -> JsErrorCode;
    pub fn JsIdle(nextIdleTick: *mut c_uint) -> JsErrorCode;
    pub fn JsGetSymbolFromPropertyId(propertyId: JsPropertyIdRef,
                                     symbol: *mut JsValueRef)
                                     -> JsErrorCode;
    pub fn JsGetPropertyIdType(propertyId: JsPropertyIdRef,
                               propertyIdType: *mut JsPropertyIdType)
                               -> JsErrorCode;
    pub fn JsGetPropertyIdFromSymbol(symbol: JsValueRef,
                                     propertyId: *mut JsPropertyIdRef)
                                     -> JsErrorCode;
    pub fn JsCreateSymbol(description: JsValueRef, result: *mut JsValueRef) -> JsErrorCode;
    pub fn JsGetOwnPropertySymbols(object: JsValueRef,
                                   propertySymbols: *mut JsValueRef)
                                   -> JsErrorCode;
    pub fn JsGetUndefinedValue(undefinedValue: *mut JsValueRef) -> JsErrorCode;
    pub fn JsGetNullValue(nullValue: *mut JsValueRef) -> JsErrorCode;
    pub fn JsGetTrueValue(trueValue: *mut JsValueRef) -> JsErrorCode;
    pub fn JsGetFalseValue(falseValue: *mut JsValueRef) -> JsErrorCode;
    pub fn JsBoolToBoolean(value: bool, booleanValue: *mut JsValueRef) -> JsErrorCode;
    pub fn JsBooleanToBool(value: JsValueRef, boolValue: *mut bool) -> JsErrorCode;
    pub fn JsConvertValueToBoolean(value: JsValueRef,
                                   booleanValue: *mut JsValueRef)
                                   -> JsErrorCode;
    pub fn JsGetValueType(value: JsValueRef, type_: *mut JsValueType) -> JsErrorCode;
    pub fn JsDoubleToNumber(doubleValue: f64, value: *mut JsValueRef) -> JsErrorCode;
    pub fn JsIntToNumber(intValue: c_int, value: *mut JsValueRef) -> JsErrorCode;
    pub fn JsNumberToDouble(value: JsValueRef, doubleValue: *mut f64) -> JsErrorCode;
    pub fn JsNumberToInt(value: JsValueRef, intValue: *mut c_int) -> JsErrorCode;
    pub fn JsConvertValueToNumber(value: JsValueRef, numberValue: *mut JsValueRef) -> JsErrorCode;
    pub fn JsGetStringLength(stringValue: JsValueRef, length: *mut c_int) -> JsErrorCode;
    pub fn JsConvertValueToString(value: JsValueRef, stringValue: *mut JsValueRef) -> JsErrorCode;
    pub fn JsGetGlobalObject(globalObject: *mut JsValueRef) -> JsErrorCode;
    pub fn JsCreateObject(object: *mut JsValueRef) -> JsErrorCode;
    pub fn JsCreateExternalObject(data: *mut c_void,
                                  finalizeCallback: JsFinalizeCallback,
                                  object: *mut JsValueRef)
                                  -> JsErrorCode;
    pub fn JsConvertValueToObject(value: JsValueRef, object: *mut JsValueRef) -> JsErrorCode;
    pub fn JsGetPrototype(object: JsValueRef, prototypeObject: *mut JsValueRef) -> JsErrorCode;
    pub fn JsSetPrototype(object: JsValueRef, prototypeObject: JsValueRef) -> JsErrorCode;
    pub fn JsInstanceOf(object: JsValueRef,
                        constructor: JsValueRef,
                        result: *mut bool)
                        -> JsErrorCode;
    pub fn JsGetExtensionAllowed(object: JsValueRef, value: *mut bool) -> JsErrorCode;
    pub fn JsPreventExtension(object: JsValueRef) -> JsErrorCode;
    pub fn JsGetProperty(object: JsValueRef,
                         propertyId: JsPropertyIdRef,
                         value: *mut JsValueRef)
                         -> JsErrorCode;
    pub fn JsGetOwnPropertyDescriptor(object: JsValueRef,
                                      propertyId: JsPropertyIdRef,
                                      propertyDescriptor: *mut JsValueRef)
                                      -> JsErrorCode;
    pub fn JsGetOwnPropertyNames(object: JsValueRef,
                                 propertyNames: *mut JsValueRef)
                                 -> JsErrorCode;
    pub fn JsSetProperty(object: JsValueRef,
                         propertyId: JsPropertyIdRef,
                         value: JsValueRef,
                         useStrictRules: bool)
                         -> JsErrorCode;
    pub fn JsHasProperty(object: JsValueRef,
                         propertyId: JsPropertyIdRef,
                         hasProperty: *mut bool)
                         -> JsErrorCode;
    pub fn JsDeleteProperty(object: JsValueRef,
                            propertyId: JsPropertyIdRef,
                            useStrictRules: bool,
                            result: *mut JsValueRef)
                            -> JsErrorCode;
    pub fn JsDefineProperty(object: JsValueRef,
                            propertyId: JsPropertyIdRef,
                            propertyDescriptor: JsValueRef,
                            result: *mut bool)
                            -> JsErrorCode;
    pub fn JsHasIndexedProperty(object: JsValueRef,
                                index: JsValueRef,
                                result: *mut bool)
                                -> JsErrorCode;
    pub fn JsGetIndexedProperty(object: JsValueRef,
                                index: JsValueRef,
                                result: *mut JsValueRef)
                                -> JsErrorCode;
    pub fn JsSetIndexedProperty(object: JsValueRef,
                                index: JsValueRef,
                                value: JsValueRef)
                                -> JsErrorCode;
    pub fn JsDeleteIndexedProperty(object: JsValueRef, index: JsValueRef) -> JsErrorCode;
    pub fn JsHasIndexedPropertiesExternalData(object: JsValueRef, value: *mut bool) -> JsErrorCode;
    pub fn JsGetIndexedPropertiesExternalData(object: JsValueRef,
                                              data: *mut *mut c_void,
                                              arrayType: *mut JsTypedArrayType,
                                              elementLength: *mut c_uint)
                                              -> JsErrorCode;
    pub fn JsSetIndexedPropertiesToExternalData(object: JsValueRef,
                                                data: *mut c_void,
                                                arrayType: JsTypedArrayType,
                                                elementLength: c_uint)
                                                -> JsErrorCode;
    pub fn JsEquals(object1: JsValueRef, object2: JsValueRef, result: *mut bool) -> JsErrorCode;
    pub fn JsStrictEquals(object1: JsValueRef,
                          object2: JsValueRef,
                          result: *mut bool)
                          -> JsErrorCode;
    pub fn JsHasExternalData(object: JsValueRef, value: *mut bool) -> JsErrorCode;
    pub fn JsGetExternalData(object: JsValueRef, externalData: *mut *mut c_void) -> JsErrorCode;
    pub fn JsSetExternalData(object: JsValueRef, externalData: *mut c_void) -> JsErrorCode;
    pub fn JsCreateArray(length: c_uint, result: *mut JsValueRef) -> JsErrorCode;
    pub fn JsCreateArrayBuffer(byteLength: c_uint, result: *mut JsValueRef) -> JsErrorCode;
    pub fn JsCreateExternalArrayBuffer(data: *mut c_void,
                                       byteLength: c_uint,
                                       finalizeCallback: JsFinalizeCallback,
                                       callbackState: *mut c_void,
                                       result: *mut JsValueRef)
                                       -> JsErrorCode;
    pub fn JsCreateTypedArray(arrayType: JsTypedArrayType,
                              baseArray: JsValueRef,
                              byteOffset: c_uint,
                              elementLength: c_uint,
                              result: *mut JsValueRef)
                              -> JsErrorCode;
    pub fn JsCreateDataView(arrayBuffer: JsValueRef,
                            byteOffset: c_uint,
                            byteLength: c_uint,
                            result: *mut JsValueRef)
                            -> JsErrorCode;
    pub fn JsGetTypedArrayInfo(typedArray: JsValueRef,
                               arrayType: *mut JsTypedArrayType,
                               arrayBuffer: *mut JsValueRef,
                               byteOffset: *mut c_uint,
                               byteLength: *mut c_uint)
                               -> JsErrorCode;
    pub fn JsGetArrayBufferStorage(arrayBuffer: JsValueRef,
                                   buffer: *mut *mut u8,
                                   bufferLength: *mut c_uint)
                                   -> JsErrorCode;
    pub fn JsGetTypedArrayStorage(typedArray: JsValueRef,
                                  buffer: *mut *mut u8,
                                  bufferLength: *mut c_uint,
                                  arrayType: *mut JsTypedArrayType,
                                  elementSize: *mut c_int)
                                  -> JsErrorCode;
    pub fn JsGetDataViewStorage(dataView: JsValueRef,
                                buffer: *mut *mut u8,
                                bufferLength: *mut c_uint)
                                -> JsErrorCode;
    pub fn JsCallFunction(function: JsValueRef,
                          arguments: *mut JsValueRef,
                          argumentCount: c_ushort,
                          result: *mut JsValueRef)
                          -> JsErrorCode;
    pub fn JsConstructObject(function: JsValueRef,
                             arguments: *mut JsValueRef,
                             argumentCount: c_ushort,
                             result: *mut JsValueRef)
                             -> JsErrorCode;
    pub fn JsCreateFunction(nativeFunction: JsNativeFunction,
                            callbackState: *mut c_void,
                            function: *mut JsValueRef)
                            -> JsErrorCode;
    pub fn JsCreateNamedFunction(name: JsValueRef,
                                 nativeFunction: JsNativeFunction,
                                 callbackState: *mut c_void,
                                 function: *mut JsValueRef)
                                 -> JsErrorCode;
    pub fn JsCreateError(message: JsValueRef, error: *mut JsValueRef) -> JsErrorCode;
    pub fn JsCreateRangeError(message: JsValueRef, error: *mut JsValueRef) -> JsErrorCode;
    pub fn JsCreateReferenceError(message: JsValueRef, error: *mut JsValueRef) -> JsErrorCode;
    pub fn JsCreateSyntaxError(message: JsValueRef, error: *mut JsValueRef) -> JsErrorCode;
    pub fn JsCreateTypeError(message: JsValueRef, error: *mut JsValueRef) -> JsErrorCode;
    pub fn JsCreateURIError(message: JsValueRef, error: *mut JsValueRef) -> JsErrorCode;
    pub fn JsHasException(hasException: *mut bool) -> JsErrorCode;
    pub fn JsGetAndClearException(exception: *mut JsValueRef) -> JsErrorCode;
    pub fn JsSetException(exception: JsValueRef) -> JsErrorCode;
    pub fn JsDisableRuntimeExecution(runtime: JsRuntimeHandle) -> JsErrorCode;
    pub fn JsEnableRuntimeExecution(runtime: JsRuntimeHandle) -> JsErrorCode;
    pub fn JsIsRuntimeExecutionDisabled(runtime: JsRuntimeHandle,
                                        isDisabled: *mut bool)
                                        -> JsErrorCode;
    pub fn JsSetPromiseContinuationCallback(promiseContinuationCallback:
                                                JsPromiseContinuationCallback,
                                            callbackState:
                                                *mut c_void)
     -> JsErrorCode;
}
