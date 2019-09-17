//! Traits for backends.

use crate::nv_tree::NativeView;
use crate::raw_events::RawEvent;

/// A backend implementation.
pub trait Backend {
    /// A reference to a view in the backend.
    type ViewRef;

    /// Error type.
    type Error;

    /// Creates a new view.
    fn new_view(&mut self, view: NativeView) -> Result<Self::ViewRef, Self::Error>;

    /// Updates the view.
    fn update(&mut self, view: &mut Self::ViewRef, view: NativeView) -> Result<(), Self::Error>;

    /// Sets a region of the view’s subviews.
    fn set_subviews<'a>(
        &mut self,
        view: &mut Self::ViewRef,
        region_start: usize,
        region_len: usize,
        subviews: Vec<&'a Self::ViewRef>,
    ) -> Result<(), Self::Error>;

    /// Returns the next event from the queue.
    ///
    /// This method may be called frequently in quick succession.
    fn poll() -> Result<Option<RawEvent>, Self::Error>;
}
