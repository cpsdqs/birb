//! Events.

use cgmath::{Point2, Vector2, Vector3};
use core::fmt;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct Event<Type> {
    data: Type,
}

/// List of event types.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventTypeId {
    Hover = 0,
    Pointer = 1,
    Key = 2,
    Scroll = 3,
}

impl EventTypeId {
    // smallest and largest values in Ord
    pub(crate) const MIN: Self = EventTypeId::Hover;
    pub(crate) const MAX: Self = EventTypeId::Scroll;
}

pub trait EventType: fmt::Debug + From<Event<Self>> {
    fn location(&self) -> Option<Point2<f64>>;
    fn type_id() -> EventTypeId;
}

/// Types of pointing devices or mechanisms.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerDevice {
    /// Touch input from a finger or something of the sort; is expected to be imprecise.
    ///
    /// Tilt will never contain useful data, but pressure may contain data from a 3D Touch
    /// screen—otherwise it’ll be constant 1.
    Touch = 0,

    /// Pen input.
    ///
    /// Tilt should default to (0, 0, 1) if it’s not supported.
    Pen = 1,

    /// Eraser input.
    ///
    /// Some erasers don’t actually support pressure at all (e.g. the Surface Pen) so in this case
    /// it should default to constant 1.
    /// Tilt should default to (0, 0, 1) if it’s not supported.
    Eraser = 2,

    /// Any indirect input mechanism.
    ///
    /// Tilt will never contain useful data, but pressure may contain data from a Force Touch
    /// trackpad—otherwise it’ll be constant 1.
    Cursor = 3,
}

impl PointerDevice {
    /// If true, the input mechanism is precise and can hit small targets.
    pub fn is_precise(&self) -> bool {
        match self {
            PointerDevice::Touch => false,
            PointerDevice::Pen | PointerDevice::Eraser | PointerDevice::Cursor => true,
        }
    }

    /// If true, the input mechanism is volatile and can’t be expected to hold perfectly still.
    ///
    /// Precise drag-and-drop is a bad experience with these when not accounted for because stuff
    /// moves a tiny bit e.g. when you lift the pen from the screen.
    pub fn is_volatile(&self) -> bool {
        match self {
            PointerDevice::Touch | PointerDevice::Pen | PointerDevice::Eraser => true,
            PointerDevice::Cursor => false,
        }
    }
}

/// A hover event.
#[derive(Debug)]
pub struct Hover {
    /// Unique ID of the pointer, or zero. If nonzero, can be expected to persist forever.
    ///
    /// This value will be computed from e.g. hardware IDs in wacom pens.
    id: u64,

    /// Event location in the parent coordinate system.
    location: Point2<f64>,

    /// Event location in the window coordinate system.
    window_location: Point2<f64>,

    /// Pointer tilt, expressed as a unit vector. Will point from the tip of the pen to the far end
    /// of the pen.
    ///
    /// The Z axis points outwards from the screen.
    tilt: Vector3<f64>,

    /// The device type that emitted this hover event.
    ///
    /// Touch devices will never emit hover events.
    device: PointerDevice,
}

impl EventType for Hover {
    fn location(&self) -> Option<Point2<f64>> {
        Some(self.location)
    }
    fn type_id() -> EventTypeId {
        EventTypeId::Hover
    }
}

impl From<Event<Hover>> for Hover {
    fn from(this: Event<Hover>) -> Self {
        this.data
    }
}

/// A pointer event.
#[derive(Debug)]
pub struct Pointer {
    /// Unique ID of the pointer, or zero. If nonzero, can be expected to persist forever.
    ///
    /// This value will be computed from e.g. hardware IDs in wacom pens.
    id: u64,

    /// Event location in the parent coordinate system.
    location: Point2<f64>,

    /// Event location in the window coordinate system.
    window_location: Point2<f64>,

    /// Pointer pressure, between 0 and 1.
    pressure: f64,

    /// Pointer tilt, expressed as a unit vector. Will point from the tip of the pen to the far end
    /// of the pen.
    ///
    /// The Z axis points outwards from the screen.
    tilt: Vector3<f64>,

    /// The device type that emitted this pointer event.
    device: PointerDevice,
}

impl EventType for Pointer {
    fn location(&self) -> Option<Point2<f64>> {
        Some(self.location)
    }
    fn type_id() -> EventTypeId {
        EventTypeId::Pointer
    }
}

impl From<Event<Pointer>> for Pointer {
    fn from(this: Event<Pointer>) -> Self {
        this.data
    }
}

/// A key event.
#[derive(Debug)]
pub struct Key {
    modifiers: KeyModifiers,
    code: KeyCode,
    // TODO
}

impl EventType for Key {
    fn location(&self) -> Option<Point2<f64>> {
        None
    }
    fn type_id() -> EventTypeId {
        EventTypeId::Key
    }
}

impl From<Event<Key>> for Key {
    fn from(this: Event<Key>) -> Self {
        this.data
    }
}

/// Modifier key state.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KeyModifiers {
    /// Whether any shift key is pressed.
    shift: bool,

    /// Whether any control key is pressed.
    control: bool,

    /// Whether any option key or alt key is pressed.
    option: bool,

    /// Whether any command key or meta key is pressed.
    command: bool,
}

/// A scroll event.
#[derive(Debug)]
pub struct Scroll {
    /// Event location in the parent coordinate system.
    location: Point2<f64>,

    /// Event location in the window coordinate system.
    window_location: Point2<f64>,

    /// Scroll delta in points.
    delta: Vector2<f64>,

    /// If true, the scrolling device is discrete (e.g. a mouse wheel that scrolls in increments)
    /// and may benefit from smooth scrolling.
    is_discrete: bool,
}

impl EventType for Scroll {
    fn location(&self) -> Option<Point2<f64>> {
        Some(self.location)
    }
    fn type_id() -> EventTypeId {
        EventTypeId::Scroll
    }
}

impl From<Event<Scroll>> for Scroll {
    fn from(this: Event<Scroll>) -> Self {
        this.data
    }
}

pub struct EventHandler<Type>(Arc<Mutex<dyn FnMut(Event<Type>) + Send>>);

impl<T> Clone for EventHandler<T> {
    fn clone(&self) -> Self {
        EventHandler(Arc::clone(&self.0))
    }
}

impl<T: EventType> EventHandler<T> {
    pub fn new<F: 'static + FnMut(Event<T>) + Send>(handler: F) -> Self {
        EventHandler(Arc::new(Mutex::new(handler)))
    }
}

impl<T: EventType> fmt::Debug for EventHandler<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EventHandler<{:?}>", T::type_id())
    }
}

/// Keyboard layout-independent identifiers for keyboard keys.
///
/// Some obscure keys may be missing.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    A = 0x1,
    B = 0x2,
    C = 0x3,
    D = 0x4,
    E = 0x5,
    F = 0x6,
    G = 0x7,
    H = 0x8,
    I = 0x9,
    J = 0xA,
    K = 0xB,
    L = 0xC,
    M = 0xD,
    N = 0xE,
    O = 0xF,
    P = 0x10,
    Q = 0x11,
    R = 0x12,
    S = 0x13,
    T = 0x14,
    U = 0x15,
    V = 0x16,
    W = 0x17,
    X = 0x18,
    Y = 0x19,
    Z = 0x1A,
    N0 = 0x20,
    N1 = 0x21,
    N2 = 0x22,
    N3 = 0x23,
    N4 = 0x24,
    N5 = 0x25,
    N6 = 0x26,
    N7 = 0x27,
    N8 = 0x28,
    N9 = 0x29,
    Equal = 0x2A,
    Minus = 0x2B,
    LeftBracket = 0x2C,
    RightBracket = 0x2D,
    Quote = 0x2E,
    Semicolon = 0x2F,
    Backslash = 0x30,
    Comma = 0x31,
    Slash = 0x32,
    Period = 0x33,
    Grave = 0x34,
    Return = 0x35,
    Tab = 0x36,
    Space = 0x37,
    Delete = 0x38,
    Escape = 0x39,
    Command = 0x3A,
    Shift = 0x3B,
    CapsLock = 0x3C,
    Option = 0x3D,
    Control = 0x3E,
    RightCommand = 0x3F,
    RightShift = 0x40,
    RightOption = 0x41,
    RightControl = 0x42,
    Function = 0x43,
    LeftArrow = 0x44,
    DownArrow = 0x45,
    UpArrow = 0x46,
    RightArrow = 0x47,
    ForwardDelete = 0x48,
    Help = 0x49,
    Home = 0x4A,
    End = 0x4B,
    PageUp = 0x4C,
    PageDown = 0x4D,
    Underscore = 0x4E,
    Section = 0x4F,
    F1 = 0x50,
    F2 = 0x51,
    F3 = 0x52,
    F4 = 0x53,
    F5 = 0x54,
    F6 = 0x55,
    F7 = 0x56,
    F8 = 0x57,
    F9 = 0x58,
    F10 = 0x59,
    F11 = 0x5A,
    F12 = 0x5B,
    F13 = 0x5C,
    F14 = 0x5D,
    F15 = 0x5E,
    F16 = 0x5F,
    F17 = 0x60,
    F18 = 0x61,
    F19 = 0x62,
    F20 = 0x63,
    Numpad0 = 0x70,
    Numpad1 = 0x71,
    Numpad2 = 0x72,
    Numpad3 = 0x73,
    Numpad4 = 0x74,
    Numpad5 = 0x75,
    Numpad6 = 0x76,
    Numpad7 = 0x77,
    Numpad8 = 0x78,
    Numpad9 = 0x79,
    NumpadEqual = 0x7A,
    NumpadDecimal = 0x7B,
    NumpadPlus = 0x7C,
    NumpadMinus = 0x7D,
    NumpadMultiply = 0x7E,
    NumpadDivide = 0x7F,
    NumpadClear = 0x80,
    NumpadEnter = 0x81,
    NumpadComma = 0x82,
}
