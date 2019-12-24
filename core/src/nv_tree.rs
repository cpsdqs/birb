use crate::backend::Backend;
use crate::color::Color;
use crate::rect::Rect;
use crate::view::{LayoutResult, ViewId};
use cgmath::Matrix3;
use core::ops::DerefMut;
use std::collections::HashMap;

#[derive(Clone)]
pub enum NativeView {
    Layer {
        bounds: Rect,
        background: Color,
        corner_radius: f64,
        border_width: f64,
        border_color: Color,
        clip_contents: bool,
        transform: Matrix3<f64>,
        opacity: f64,
    },
}

/// Patches for the NV tree.
#[derive(Clone)]
pub enum Patch {
    /// Sets the root view.
    SetRoot(ViewId),
    /// Updates or creates a view.
    Update(ViewId, NativeView),
    /// Deletes and re-creates a view.
    Replace(ViewId, NativeView),
    /// Replaces a region of a view’s subviews.
    ///
    /// `(superview, region, subviews)`
    SubviewRegion(ViewId, usize, usize, Vec<ViewId>),
    /// Removes a view.
    /// **Does not remove the view from the superview’s subview references.**
    Remove(ViewId),
}

/// Errors that may occur when running a patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PatchError<B: Backend> {
    NoSuchView(ViewId),
    BackendError(B::Error),
    Cycle(ViewId),
}

struct NVTNode<R> {
    view: NativeView,
    backing_ref: R,
    superview: Option<ViewId>,
    subviews: Vec<ViewId>,
    layout: Option<LayoutResult>,
}

/// The native-view tree; handles layout, events, and backends.
pub struct NVTree<B, R> {
    nodes: HashMap<ViewId, NVTNode<R>>,
    backend: B,
    // TODO: spatial index
    tracking_rects: HashMap<ViewId, Rect>,
}

impl<B: DerefMut<Target = Bknd>, Bknd: Backend> NVTree<B, Bknd::ViewRef> {
    pub fn new(backend: B) -> NVTree<B, Bknd::ViewRef> {
        NVTree {
            nodes: HashMap::new(),
            backend,
            tracking_rects: HashMap::new(),
        }
    }

    /// Patches the view tree.
    pub fn patch(&mut self, patch: Patch) -> Result<(), PatchError<Bknd>> {
        match patch {
            Patch::SetRoot(id) => self.set_root(id),
            Patch::Update(id, view) => self.update_view(id, view, None),
            Patch::Replace(id, view) => self.replace_view(id, view),
            Patch::SubviewRegion(id, a, b, subviews) => self.subview_region(id, a, b, subviews),
            Patch::Remove(id) => self.remove_view(id, true).map(|_| ()),
        }
    }

    /// Sets a root view.
    fn set_root(&mut self, id: ViewId) -> Result<(), PatchError<Bknd>> {
        if let Some(node) = self.nodes.get_mut(&id) {
            self.backend.set_root_view(&mut node.backing_ref).map_err(PatchError::BackendError)?;
            Ok(())
        } else {
            Err(PatchError::NoSuchView(id))
        }
    }

    /// Updates or adds a view.
    fn update_view(
        &mut self,
        id: ViewId,
        view: NativeView,
        bref: Option<Bknd::ViewRef>,
    ) -> Result<(), PatchError<Bknd>> {
        if let Some(node) = self.nodes.get_mut(&id) {
            self.backend
                .update_view(&mut node.backing_ref, view.clone())
                .map_err(PatchError::BackendError)?;
            node.view = view;
        } else {
            let backing_ref = if let Some(bref) = bref {
                bref
            } else {
                self.backend
                    .new_view(view.clone())
                    .map_err(PatchError::BackendError)?
            };
            self.nodes.insert(
                id,
                NVTNode {
                    view,
                    backing_ref,
                    superview: None,
                    subviews: Vec::new(),
                    layout: None,
                },
            );
        }
        Ok(())
    }

    fn replace_view(&mut self, id: ViewId, view: NativeView) -> Result<(), PatchError<Bknd>> {
        let backing_ref = self
            .remove_view(id, false)?
            .expect("remove_view should have returned a backing ref if dispatch is false");
        self.update_view(id, view, Some(backing_ref))
    }

    /// Does not remove the view from the superview’s subviews list.
    fn remove_view(
        &mut self,
        id: ViewId,
        dispatch: bool,
    ) -> Result<Option<Bknd::ViewRef>, PatchError<Bknd>> {
        if let Some(node) = self.nodes.remove(&id) {
            for id in node.subviews {
                self.remove_view(id, true)?;
            }
            if dispatch {
                self.backend
                    .remove_view(node.backing_ref)
                    .map_err(PatchError::BackendError)?;
                Ok(None)
            } else {
                Ok(Some(node.backing_ref))
            }
        } else {
            Err(PatchError::NoSuchView(id))
        }
    }

    /// # Panics
    /// - if the view is its own direct descendant
    fn subview_region(
        &mut self,
        id: ViewId,
        offset: usize,
        len: usize,
        subviews: Vec<ViewId>,
    ) -> Result<(), PatchError<Bknd>> {
        // set the superview property of all subviews
        for subview in &subviews {
            let node = match self.nodes.get_mut(subview) {
                Some(node) => node,
                None => return Err(PatchError::NoSuchView(*subview)),
            };
            node.superview = Some(id);
        }

        // remove the superview node because we need to alias self.nodes when sending a message to
        // the backend
        let mut superview_node = match self.nodes.remove(&id) {
            Some(node) => node,
            None => return Err(PatchError::NoSuchView(id)),
        };

        // send a message to the backend
        {
            let superview_ref = &mut superview_node.backing_ref;

            let mut subview_refs = Vec::with_capacity(subviews.len());
            for id in &subviews {
                match self.nodes.get(&id) {
                    Some(node) => subview_refs.push(&node.backing_ref),
                    None => {
                        // there are two ways to get here:
                        // either we have weird data races, considering we checked existence of all
                        // these subviews in the loop at the beginning of this function, or the
                        // superview is in the subviews list.
                        // We’ll assume that the second case has happened because safety invariants.
                        return Err(PatchError::Cycle(*id));
                    }
                }
            }

            self.backend
                .set_subviews(superview_ref, offset, len, subview_refs)
                .map_err(PatchError::BackendError)?;
        }

        // update our own subview list
        {
            // superview_node.subviews[offset..len] = subviews[..len]
            for (i, j) in (offset..len).zip(0..subviews.len()) {
                superview_node.subviews[i] = subviews[j];
            }
            if subviews.len() < len {
                // superview_node.subviews[offset + subviews.len()..len] = []
                for _ in subviews.len()..len {
                    superview_node.subviews.remove(offset + subviews.len());
                }
            }
            if subviews.len() > len {
                // superview_node.subviews[offset + len] <- subviews[len..]
                for i in len..subviews.len() {
                    superview_node
                        .subviews
                        .insert(offset + subviews.len(), subviews[i]);
                }
            }
        }

        self.nodes.insert(id, superview_node);
        Ok(())
    }
}
