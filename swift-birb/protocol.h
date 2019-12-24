#ifndef BIRB_H
#define BIRB_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#pragma mark - Basic Data Types

typedef double float64_t;

/** A two-dimensional vector or point. */
typedef struct {
    float64_t x;
    float64_t y;
} SBVector2;

/** A three-dimensional vector or point. */
typedef struct {
    float64_t x;
    float64_t y;
    float64_t z;
} SBVector3;

/** A three-dimensional transformation matrix. */
typedef struct {
    float64_t m00;
    float64_t m01;
    float64_t m02;
    float64_t m10;
    float64_t m11;
    float64_t m12;
    float64_t m20;
    float64_t m21;
    float64_t m22;
} SBMatrix3;

/** A rectangle. */
typedef struct {
    SBVector2 origin;
    SBVector2 size;
} SBRect;

/** An RGBA color. */
typedef struct {
    float64_t r;
    float64_t g;
    float64_t b;
    float64_t a;
} SBColor;

/**
 * A unique identifier for a view.
 *
 * (This is just a UUID)
 */
typedef struct {
    uint32_t a;
    uint16_t b;
    uint16_t c;
    uint8_t d[8];
} SBViewId;

#pragma mark - Events

/** Types of events. */
typedef enum SBEventTypeId {
    SBEventTypeIdHover = 0,
    SBEventTypeIdPointer = 1,
    SBEventTypeIdKey = 2,
    SBEventTypeIdScroll = 3,
    SBEventTypeIdResize = 4,
} SBEventTypeId;

/** A unique identifier for an event handler. */
typedef struct {
    SBViewId view;
    SBEventTypeId type;
} SBHandlerId;

/** Keyboard modifiers. */
typedef struct {
    /** Whether the shift key is being pressed. */
    bool shift;
    /** Whether the control key is being pressed. */
    bool control;
    /** Whether the option key (a.k.a. alt key) is being pressed. */
    bool option;
    /** Whether the command key (a.k.a. meta key) is being pressed. */
    bool command;
} SBKeyModifiers;

/** Types of pointing devices. */
typedef enum SBPointerDevice {
    SBPointerDeviceTouch = 0,
    SBPointerDevicePen = 1,
    SBPointerDeviceEraser = 2,
    SBPointerDeviceCursor = 3,
} SBPointerDevice;

/** Type of unique pointer IDs. */
typedef uint64_t SBPointerID;

/**
 * Hover event phases.
 *
 * This enum has an ordering: Entered < Moved = Stationary < Left, and events are guaranteed
 * to be generated in this order for a given device.
 */
typedef enum SBHoverEventPhase {
    /**
     * The device has entered proximity.
     *
     * This event *must* be emitted before any other hover event for a device, even if the device
     * does not support the notion of proximity (such as a mouse or trackpad).
     */
    SBHoverEventPhaseEntered = 0,
    /** The device has been moved since the last event. */
    SBHoverEventPhaseMoved = 1,
    /**
     * The device hasn’t moved since the last event but a hover event is being emitted anyway,
     * most likely caused by a change in tilt or other lateral parameters.
     */
    SBHoverEventPhaseStationary = 2,
    /** The device has left proximity. */
    SBHoverEventPhaseLeft = 3,
} SBHoverEventPhase;

/** Hover events. */
typedef struct {
    /** The kind of device that is generating hover events. */
    SBPointerDevice device;
    /** The location in the window. */
    SBVector2 window_location;
    /**
     * The device’s tilt, expressed as a unit vector aligned with the window coordinate system, with
     * an additional Z axis pointing outwards.
     *
     * Devices that do not support this should always have a tilt of [0, 1, 1].
     */
    SBVector3 tilt;
    /**
     * The unique ID of the pointing device that generated this event; may be zero.
     *
     * If nonzero, it is guaranteed to be stable.
     */
    SBPointerID pointer_id;
    /** The hover event phase for this pointing device. */
    SBHoverEventPhase phase;
    /** The modifier keys that are currently being pressed. */
    SBKeyModifiers modifiers;
    // TODO: NSEvent.ButtonMask
} SBHoverEvent;

/**
 * Pointer event phases.
 *
 * This enum has an ordering: Began < Moved = Stationary < Ended = Canceled, and events are
 * guaranteed to be generated in this order for a given device.
 */
typedef enum SBPointerEventPhase {
    /**
     * The pointing device has been activated.
     *
     * This usually means that the device has touched down on the screen, or that a mouse button
     * was pressed. This will not be emitted if more mouse buttons are pressed.
     */
    SBPointerEventPhaseBegan = 0,
    /** The pointing device has been moved since the last event. */
    SBPointerEventPhaseMoved = 1,
    /**
     * The pointing device has not been moved since the last event, but an event is being emitted
     * anyway, probably caused by lateral properties such as pressure or tilt.
     */
    SBPointerEventPhaseStationary = 2,
    /**
     * The pointing device has been completely deactivated.
     *
     * This usually means that the device has stopped touching the screen, or that all mouse buttons
     * have been released.
     */
    SBPointerEventPhaseEnded = 3,
    /** The stream of pointer events has been canceled for some reason. */
    SBPointerEventPhaseCanceled = 4,
} SBPointerEventPhase;

/** Pointer events. */
typedef struct {
    /** The kind of device that is generating pointer events. */
    SBPointerDevice device;
    /** The location in the window. */
    SBVector2 window_location;
    /**
     * The pressure with which the device may be pressing down on the screen.
     *
     * Will be 1 for devices that do not support pressure.
     */
    float64_t pressure;
    /**
     * The device’s tilt, expressed as a unit vector aligned with the window coordinate system, with
     * an additional Z axis pointing outwards.
     *
     * Devices that do not support this should always have a tilt of [0, 1, 1].
     */
    SBVector3 tilt;
    /**
     * The unique ID of the pointing device that generated this event; may be zero.
     *
     * If nonzero, it is guaranteed to be stable.
     */
    SBPointerID pointer_id;
    /** The pointer event phase for this pointing device. */
    SBPointerEventPhase phase;
    /** The modifier keys that are currently being pressed. */
    SBKeyModifiers modifiers;
} SBPointerEvent;

/** Key codes. */
typedef enum SBKeyCode {
    SBKeyCodeA = 0x1,
    SBKeyCodeB = 0x2,
    SBKeyCodeC = 0x3,
    SBKeyCodeD = 0x4,
    SBKeyCodeE = 0x5,
    SBKeyCodeF = 0x6,
    SBKeyCodeG = 0x7,
    SBKeyCodeH = 0x8,
    SBKeyCodeI = 0x9,
    SBKeyCodeJ = 0xA,
    SBKeyCodeK = 0xB,
    SBKeyCodeL = 0xC,
    SBKeyCodeM = 0xD,
    SBKeyCodeN = 0xE,
    SBKeyCodeO = 0xF,
    SBKeyCodeP = 0x10,
    SBKeyCodeQ = 0x11,
    SBKeyCodeR = 0x12,
    SBKeyCodeS = 0x13,
    SBKeyCodeT = 0x14,
    SBKeyCodeU = 0x15,
    SBKeyCodeV = 0x16,
    SBKeyCodeW = 0x17,
    SBKeyCodeX = 0x18,
    SBKeyCodeY = 0x19,
    SBKeyCodeZ = 0x1A,
    SBKeyCodeN0 = 0x20,
    SBKeyCodeN1 = 0x21,
    SBKeyCodeN2 = 0x22,
    SBKeyCodeN3 = 0x23,
    SBKeyCodeN4 = 0x24,
    SBKeyCodeN5 = 0x25,
    SBKeyCodeN6 = 0x26,
    SBKeyCodeN7 = 0x27,
    SBKeyCodeN8 = 0x28,
    SBKeyCodeN9 = 0x29,
    SBKeyCodeEqual = 0x2A,
    SBKeyCodeMinus = 0x2B,
    SBKeyCodeLeftBracket = 0x2C,
    SBKeyCodeRightBracket = 0x2D,
    SBKeyCodeQuote = 0x2E,
    SBKeyCodeSemicolon = 0x2F,
    SBKeyCodeBackslash = 0x30,
    SBKeyCodeComma = 0x31,
    SBKeyCodeSlash = 0x32,
    SBKeyCodePeriod = 0x33,
    SBKeyCodeGrave = 0x34,
    SBKeyCodeReturn = 0x35,
    SBKeyCodeTab = 0x36,
    SBKeyCodeSpace = 0x37,
    SBKeyCodeDelete = 0x38,
    SBKeyCodeEscape = 0x39,
    SBKeyCodeCommand = 0x3A,
    SBKeyCodeShift = 0x3B,
    SBKeyCodeCapsLock = 0x3C,
    SBKeyCodeOption = 0x3D,
    SBKeyCodeControl = 0x3E,
    SBKeyCodeRightCommand = 0x3F,
    SBKeyCodeRightShift = 0x40,
    SBKeyCodeRightOption = 0x41,
    SBKeyCodeRightControl = 0x42,
    SBKeyCodeFunction = 0x43,
    SBKeyCodeLeftArrow = 0x44,
    SBKeyCodeDownArrow = 0x45,
    SBKeyCodeUpArrow = 0x46,
    SBKeyCodeRightArrow = 0x47,
    SBKeyCodeForwardDelete = 0x48,
    SBKeyCodeInsert = 0x49,
    SBKeyCodeHome = 0x4A,
    SBKeyCodeEnd = 0x4B,
    SBKeyCodePageUp = 0x4C,
    SBKeyCodePageDown = 0x4D,
    SBKeyCodeSection = 0x4E,
    SBKeyCodeF1 = 0x50,
    SBKeyCodeF2 = 0x51,
    SBKeyCodeF3 = 0x52,
    SBKeyCodeF4 = 0x53,
    SBKeyCodeF5 = 0x54,
    SBKeyCodeF6 = 0x55,
    SBKeyCodeF7 = 0x56,
    SBKeyCodeF8 = 0x57,
    SBKeyCodeF9 = 0x58,
    SBKeyCodeF10 = 0x59,
    SBKeyCodeF11 = 0x5A,
    SBKeyCodeF12 = 0x5B,
    SBKeyCodeF13 = 0x5C,
    SBKeyCodeF14 = 0x5D,
    SBKeyCodeF15 = 0x5E,
    SBKeyCodeF16 = 0x5F,
    SBKeyCodeF17 = 0x60,
    SBKeyCodeF18 = 0x61,
    SBKeyCodeF19 = 0x62,
    SBKeyCodeF20 = 0x63,
    SBKeyCodeNumpad0 = 0x70,
    SBKeyCodeNumpad1 = 0x71,
    SBKeyCodeNumpad2 = 0x72,
    SBKeyCodeNumpad3 = 0x73,
    SBKeyCodeNumpad4 = 0x74,
    SBKeyCodeNumpad5 = 0x75,
    SBKeyCodeNumpad6 = 0x76,
    SBKeyCodeNumpad7 = 0x77,
    SBKeyCodeNumpad8 = 0x78,
    SBKeyCodeNumpad9 = 0x79,
    SBKeyCodeNumpadEqual = 0x7A,
    SBKeyCodeNumpadDecimal = 0x7B,
    SBKeyCodeNumpadPlus = 0x7C,
    SBKeyCodeNumpadMinus = 0x7D,
    SBKeyCodeNumpadMultiply = 0x7E,
    SBKeyCodeNumpadDivide = 0x7F,
    SBKeyCodeNumpadClear = 0x80,
    SBKeyCodeNumpadEnter = 0x81,
    SBKeyCodeNumpadComma = 0x82,
} SBKeyCode;

/**
 * Keyboard event phases.
 *
 * This enum has an ordering: Down < Repeat < Up, and events are guaranteed to be generated in this
 * order for any given key.
 */
typedef enum SBKeyEventPhase {
    SBKeyEventPhaseDown = 0,
    SBKeyEventPhaseRepeat = 2,
    SBKeyEventPhaseUp = 1,
} SBKeyEventPhase;

/** Keyboard events. */
typedef struct {
    /** The characters that are being input. */
    char* chars;
    /** The characters that would be input were the modifier keys not being pressed. */
    char* chars_without_mod;
    /** The key code of the key being pressed or released. */
    SBKeyCode keyCode;
    /** The phase of this keyboard event. */
    SBKeyEventPhase phase;
    /** The modifier keys that are currently being pressed. */
    SBKeyModifiers modifiers;
} SBKeyEvent;

/** Scroll events. */
typedef struct {
    /** The location in the window. */
    SBVector2 window_location;
    /** The scroll delta, in points. */
    SBVector2 delta;
} SBScrollEvent;

/** Event data. */
typedef union {
    SBHoverEvent hover;
    SBPointerEvent pointer;
    SBKeyEvent key;
    SBScrollEvent scroll;
} SBEventData;

/** An event. */
typedef struct {
    SBEventTypeId type;
    SBHandlerId handler;
    /**
     * The timestamp of the event, in seconds, starting at some fixed point.
     * May be zero if it doesn’t have one.
     */
    float64_t timestamp;
    SBEventData data;
} SBEvent;

/** The receiver function for events. */
typedef void (*SBEventDispatcher)(SBEvent event, size_t user_data);

#pragma mark - Patches

/** Patch types. */
typedef enum SBPatchType {
    /** Updates or creates a view. */
    SBPatchTypeUpdate = 0,
    /** Sets up a superview-subview relationship. */
    SBPatchTypeSubview = 1,
    /** Removes a view and its subviews. */
    SBPatchTypeRemove = 2,
} SBPatchType;

/** Layer description. */
typedef struct {
    SBRect bounds;
    SBColor background;
    float64_t corner_radius;
    float64_t border_width;
    SBColor border_color;
    bool clip_contents;
    SBMatrix3 transform;
    float64_t opacity;
} SBLayerPatch;

/** Types of nodes. */
typedef enum SBNodeType {
    SBNodeTypeLayer = 0,
    SBNodeTypeText = 1,
    SBNodeTypeTextField = 2,
    SBNodeTypeVkSurface = 3,
} SBNodeType;

/** Update patch data. */
typedef union {
    SBLayerPatch layer;
} SBNodePatchData;

/** An update patch. */
typedef struct {
    SBNodeType type;
    SBNodePatchData patch;
} SBNodePatch;

/** Patch data. */
typedef union {
    SBNodePatch update;
    SBViewId subview;
} SBPatchData;

/** A patch. */
typedef struct {
    SBPatchType type;
    SBViewId view;
    SBPatchData data;
} SBPatch;

/** A list of nodes. Ownership of the pointers is guaranteed (not the pointees though) */
typedef struct {
    void* nodes;
    uint64_t count;
} SBNodeList;

#endif // BIRB_H
