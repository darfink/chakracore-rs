use libc::{c_char, c_int, c_uint, c_void};
use common::*;

/// Debug events reported from ChakraCore engine.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum JsDiagDebugEvent {
    /// Indicates a new script being compiled, this includes script, eval, new
    /// function.
    SourceCompile = 0,
    /// Indicates compile error for a script.
    CompileError = 1,
    /// Indicates a break due to a breakpoint.
    Breakpoint = 2,
    /// Indicates a break after completion of step action.
    StepComplete = 3,
    /// Indicates a break due to debugger statement.
    DebuggerStatement = 4,
    /// Indicates a break due to async break.
    AsyncBreak = 5,
    /// Indicates a break due to a runtime script exception.
    RuntimeException = 6,
}

bitflags! {
    /// Break on Exception attributes.
    pub flags JsDiagBreakOnExceptionAttributes: c_int {
        /// Don't break on any exception.
        const JsDiagBreakOnExceptionAttributeNone = 0,
        /// Break on uncaught exception.
        const JsDiagBreakOnExceptionAttributeUncaught = (1 << 0),
        /// Break on first chance exception.
        const JsDiagBreakOnExceptionAttributeFirstChance = (1 << 1),
    }
}

/// Stepping types.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum JsDiagStepType {
    /// Perform a step operation to next statement.
    In = 0,
    /// Perform a step out from the current function.
    Out = 1,
    /// Perform a single step over after a debug break if the next statement is
    /// a function call, else behaves as a stepin.
    Over = 2,
    /// Perform a single step back to the previous statement (only applicable in
    /// TTD mode).
    Back = 3,
    /// Perform a reverse continue operation (only applicable in TTD mode).
    ReverseContinue = 4,
}

/// User implemented callback routine for debug events.
/// Use `JsDiagStartDebugging` to register the callback.
///
/// - `debugEvent`: The type of JsDiagDebugEvent event.
/// - `eventData`: Additional data related to the debug event.
/// - `callbackState`: The state passed to `JsDiagStartDebugging`.
pub type JsDiagDebugEventCallback = Option<extern "system" fn(debugEvent: JsDiagDebugEvent,
                                                              eventData: JsValueRef,
                                                              callbackState: *mut c_void)>;

bitflags! {
    /// TimeTravel move options as bit flag enum.
    #[allow(improper_ctypes)]
    pub flags JsTTDMoveMode : i64 {
        /// Indicates no special actions needed for move.
        const JsTTDMoveNone = 0,
        /// Indicates that we want to move to the first event.
        const JsTTDMoveFirstEvent = (1 << 0),
        /// Indicates that we want to move to the last event.
        const JsTTDMoveLastEvent = (1 << 1),
        /// Indicates that we want to move to the kth event -- top 32 bits are
        /// event count.
        const JsTTDMoveKthEvent = (1 << 2),
        /// Indicates if we are doing the scan for a continue operation
        const JsTTDMoveScanIntervalForContinue = (1 << 4),
        /// Indicates if we want to set break on entry or just run and let
        /// something else trigger breakpoints.
        const JsTTDMoveBreakOnEntry = (1 << 8),
    }
}

/// A handle for URI's that TTD information is written to/read from.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct JsTTDStreamHandle(pub *mut c_void);

pub type JsTTDInitializeForWriteLogStreamCallback =
    Option<extern "system" fn(uriByteLength: usize, uriBytes: *const u8)>;

pub type TTDOpenResourceStreamCallback =
    Option<extern "system" fn(uriByteLength: usize,
                              uriBytes: *const u8,
                              asciiResourceName: *const c_char,
                              read: bool,
                              write: bool,
                              relocatedUri: *mut *mut u8,
                              relocatedUriLength: *mut usize)
                              -> JsTTDStreamHandle>;

pub type JsTTDReadBytesFromStreamCallback = Option<extern "system" fn(handle: JsTTDStreamHandle,
                                                                      buff: *mut u8,
                                                                      size: usize,
                                                                      readCount: *mut usize)
                                                                      -> bool>;

pub type JsTTDWriteBytesToStreamCallback = Option<extern "system" fn(handle: JsTTDStreamHandle,
                                                                     buff: *const u8,
                                                                     size: usize,
                                                                     writtenCount: *mut usize)
                                                                     -> bool>;

pub type JsTTDFlushAndCloseStreamCallback = Option<extern "system" fn(handle: JsTTDStreamHandle,
                                                                      read: bool,
                                                                      write: bool)>;

extern "system" {
    pub fn JsDiagStartDebugging(runtimeHandle: JsRuntimeHandle,
                                debugEventCallback: JsDiagDebugEventCallback,
                                callbackState: *mut c_void)
                                -> JsErrorCode;
    pub fn JsDiagStopDebugging(runtimeHandle: JsRuntimeHandle,
                               callbackState: *mut *mut c_void)
                               -> JsErrorCode;
    pub fn JsDiagRequestAsyncBreak(runtimeHandle: JsRuntimeHandle) -> JsErrorCode;
    pub fn JsDiagGetBreakpoints(breakpoints: *mut JsValueRef) -> JsErrorCode;
    pub fn JsDiagSetBreakpoint(scriptId: c_uint,
                               lineNumber: c_uint,
                               columnNumber: c_uint,
                               breakpoint: *mut JsValueRef)
                               -> JsErrorCode;
    pub fn JsDiagRemoveBreakpoint(breakpointId: c_uint) -> JsErrorCode;
    pub fn JsDiagSetBreakOnException(runtimeHandle: JsRuntimeHandle,
                                     exceptionAttributes: JsDiagBreakOnExceptionAttributes)
                                     -> JsErrorCode;
    pub fn JsDiagGetBreakOnException(runtimeHandle: JsRuntimeHandle,
                                     exceptionAttributes: *mut JsDiagBreakOnExceptionAttributes)
                                     -> JsErrorCode;
    pub fn JsDiagSetStepType(stepType: JsDiagStepType) -> JsErrorCode;
    pub fn JsDiagGetScripts(scriptsArray: *mut JsValueRef) -> JsErrorCode;
    pub fn JsDiagGetSource(scriptId: c_uint, source: *mut JsValueRef) -> JsErrorCode;
    pub fn JsDiagGetFunctionPosition(function: JsValueRef,
                                     functionPosition: *mut JsValueRef)
                                     -> JsErrorCode;
    pub fn JsDiagGetStackTrace(stackTrace: *mut JsValueRef) -> JsErrorCode;
    pub fn JsDiagGetStackProperties(stackFrameIndex: c_uint,
                                    properties: *mut JsValueRef)
                                    -> JsErrorCode;
    pub fn JsDiagGetProperties(objectHandle: c_uint,
                               fromCount: c_uint,
                               totalCount: c_uint,
                               propertiesObject: *mut JsValueRef)
                               -> JsErrorCode;
    pub fn JsDiagGetObjectFromHandle(objectHandle: c_uint,
                                     handleObject: *mut JsValueRef)
                                     -> JsErrorCode;
    #[cfg(windows)]
    pub fn JsDiagEvaluate(expression: *mut *const wchar_t,
                          stackFrameIndex: c_uint,
                          evalResult: *mut JsValueRef);
    pub fn JsDiagEvaluateUtf8(expression: *const c_char,
                              stackFrameIndex: c_uint,
                              evalResult: *mut JsValueRef)
                              -> JsErrorCode;
    pub fn JsTTDCreateRecordRuntime(attributes: JsRuntimeAttributes,
                                    infoUri: *const u8,
                                    infoUriCount: usize,
                                    snapInterval: usize,
                                    snapHistoryLength: usize,
                                    writeInitializeFunction:
                                        JsTTDInitializeForWriteLogStreamCallback,
                                    openResourceStream:
                                        TTDOpenResourceStreamCallback,
                                    readBytesFromStream:
                                        JsTTDReadBytesFromStreamCallback,
                                    writeBytesToStream:
                                        JsTTDWriteBytesToStreamCallback,
                                    flushAndCloseStream:
                                        JsTTDFlushAndCloseStreamCallback,
                                    threadService: JsThreadServiceCallback,
                                    runtime: *mut JsRuntimeHandle)
     -> JsErrorCode;
    pub fn JsTTDCreateReplayRuntime(attributes: JsRuntimeAttributes,
                                    infoUri: *const u8,
                                    infoUriCount: usize, enableDebugging: bool,
                                    writeInitializeFunction:
                                        JsTTDInitializeForWriteLogStreamCallback,
                                    openResourceStream:
                                        TTDOpenResourceStreamCallback,
                                    readBytesFromStream:
                                        JsTTDReadBytesFromStreamCallback,
                                    writeBytesToStream:
                                        JsTTDWriteBytesToStreamCallback,
                                    flushAndCloseStream:
                                        JsTTDFlushAndCloseStreamCallback,
                                    threadService: JsThreadServiceCallback,
                                    runtime: *mut JsRuntimeHandle)
     -> JsErrorCode;
    pub fn JsTTDCreateContext(runtimeHandle: JsRuntimeHandle,
                              useRuntimeTTDMode: bool,
                              newContext: *mut JsContextRef)
                              -> JsErrorCode;
    pub fn JsTTDNotifyContextDestroy(context: JsContextRef) -> JsErrorCode;
    pub fn JsTTDStart() -> JsErrorCode;
    pub fn JsTTDStop() -> JsErrorCode;
    pub fn JsTTDEmitRecording() -> JsErrorCode;
    pub fn JsTTDPauseTimeTravelBeforeRuntimeOperation() -> JsErrorCode;
    pub fn JsTTDReStartTimeTravelAfterRuntimeOperation() -> JsErrorCode;
    pub fn JsTTDNotifyYield() -> JsErrorCode;
    pub fn JsTTDHostExit(statusCode: c_int) -> JsErrorCode;
    pub fn JsTTDRawBufferCopySyncIndirect(dst: JsValueRef,
                                          dstIndex: usize,
                                          src: JsValueRef,
                                          srcIndex: usize,
                                          count: usize)
                                          -> JsErrorCode;
    pub fn JsTTDRawBufferModifySyncIndirect(buffer: JsValueRef,
                                            index: usize,
                                            count: usize)
                                            -> JsErrorCode;
    pub fn JsTTDRawBufferAsyncModificationRegister(instance: JsValueRef,
                                                   initialModPos: *mut u8)
                                                   -> JsErrorCode;
    pub fn JsTTDRawBufferAsyncModifyComplete(finalModPos: *mut u8) -> JsErrorCode;
    pub fn JsTTDCheckAndAssertIfTTDRunning(msg: *const c_char) -> JsErrorCode;
    pub fn JsTTDGetSnapTimeTopLevelEventMove(runtimeHandle: JsRuntimeHandle,
                                             moveMode: JsTTDMoveMode,
                                             targetEventTime: *mut i64,
                                             targetStartSnapTime: *mut i64,
                                             targetEndSnapTime: *mut i64)
                                             -> JsErrorCode;
    pub fn JsTTDGetSnapShotBoundInterval(runtimeHandle: JsRuntimeHandle,
                                         targetEventTime: i64,
                                         startSnapTime: *mut i64,
                                         endSnapTime: *mut i64)
                                         -> JsErrorCode;
    pub fn JsTTDGetPreviousSnapshotInterval(runtimeHandle: JsRuntimeHandle,
                                            currentSnapStartTime: i64,
                                            previousSnapTime: *mut i64)
                                            -> JsErrorCode;
    pub fn JsTTDPreExecuteSnapShotInterval(runtimeHandle: JsRuntimeHandle,
                                           startSnapTime: i64,
                                           endSnapTime: i64,
                                           moveMode: JsTTDMoveMode,
                                           newTargetEventTime: *mut i64)
                                           -> JsErrorCode;
    pub fn JsTTDMoveToTopLevelEvent(runtimeHandle: JsRuntimeHandle,
                                    moveMode: JsTTDMoveMode,
                                    snapshotTime: i64,
                                    eventTime: i64)
                                    -> JsErrorCode;
    pub fn JsTTDReplayExecution(moveMode: *mut JsTTDMoveMode,
                                rootEventTime: *mut i64)
                                -> JsErrorCode;
}
