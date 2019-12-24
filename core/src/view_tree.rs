use crate::nv_tree::Patch;
use crate::view::{Fragment, State, View, ViewId};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Clone, Copy)]
struct Subregion {
    pos: usize,
    len: usize,
}

/// A node in the view tree.
struct TreeNode<Ctx> {
    /// The current view object.
    view: Arc<dyn View<Ctx>>,
    /// If true, this view is a native view.
    is_native: bool,
    /// The immediate superview.
    superview: Option<ViewId>,
    /// The closest ancestor that is a native view.
    nv_ancestor: Option<ViewId>,
    /// Subregion in the native ancestor’s subviews.
    /// This is because one composite view may have multiple native subviews, so we need to know
    /// how many of them and where they are in the native view tree.
    nv_subregion: Subregion,
    /// The view state.
    state: Box<dyn State<Ctx>>,
    /// An ordered list of all subviews.
    subviews: Vec<ViewId>,
    /// The node’s inherited context.
    context: Ctx,
}

/// A view tree; contains a hierarchy of virtual views and manages rendering and updating.
pub struct ViewTree<Ctx> {
    nodes: HashMap<ViewId, TreeNode<Ctx>>,
    root: Option<ViewId>,
    patches: VecDeque<Patch>,
}

/// A view’s context.
#[derive(Debug)]
pub struct Context<Ctx> {
    // TODO
    context: Ctx,
}

impl<Ctx> Context<Ctx> {
    pub fn request_render(&self) {
        todo!()
    }

    pub fn request_layout(&self) {
        todo!()
    }

    pub fn request_context(&self) {
        // FIXME: what is this??
        todo!()
    }

    pub fn ctx(&self) -> &Ctx {
        &self.context
    }
}

impl<Ctx: 'static> ViewTree<Ctx>
where
    Ctx: Clone + Send,
{
    pub fn new() -> ViewTree<Ctx> {
        ViewTree {
            nodes: HashMap::new(),
            root: None,
            patches: VecDeque::new(),
        }
    }

    /// Returns an iterator over available patches.
    ///
    /// Does not drain the queue immediately.
    /// Calling `next` will always remove a patch from the queue.
    pub fn patches(&mut self) -> impl Iterator<Item = Patch> + '_ {
        struct PatchIterator<'a, T>(&'a mut ViewTree<T>);
        impl<'a, T> Iterator for PatchIterator<'a, T> {
            type Item = Patch;
            fn next(&mut self) -> Option<Patch> {
                self.0.patches.pop_front()
            }
        }

        PatchIterator(self)
    }

    /// Renders a root view.
    pub fn render_root(&mut self, view: Arc<dyn View<Ctx>>, context: Ctx) {
        if let Some(root) = self.root {
            self.diff(root, &view, 0, context);
        } else {
            let root_id = ViewId::new();
            self.root = Some(root_id);
            self.diff(root_id, &view, 0, context);
            self.patches.push_back(Patch::SetRoot(root_id));
        }
    }

    /// Diffs a view with its current state in the tree.
    ///
    /// - `id`: the view id, for identifying the tree node
    /// - `view`: the new view
    /// - `nv_subregion_start`: the start index for the NV subregion for this view
    ///
    /// Returns native view IDs that are descendants of this view.
    fn diff(
        &mut self,
        id: ViewId,
        view: &Arc<dyn View<Ctx>>,
        nv_subregion_start: usize,
        context: Ctx,
    ) -> Vec<ViewId> {
        if let Some(node) = self.nodes.get(&id) {
            let mut is_same_type = node.view.as_any().type_id() == view.as_any().type_id();
            if is_same_type {
                // allow proxy views to complain if they’re not actually the same type
                if !node.view.is_same_type(&**view) {
                    is_same_type = false;
                } else {
                    // same type; can be diffed
                    if !node.view.eq(&**view) {
                        self.update_view(id, view);
                    }
                }
            }

            if !is_same_type {
                // different type; needs to be replaced
                self.replace_view(id, view, nv_subregion_start, context);
            }
        } else {
            // does not exist; needs to be added
            self.add_view(id, view, nv_subregion_start, context);
        }

        // render the node’s body
        let node = self.nodes.get_mut(&id).unwrap();
        let body = node.view.body(&node.state);
        let subview_subregion_start = if node.is_native {
            0
        } else {
            nv_subregion_start
        };
        let subviews = self.diff_subviews(id, body, subview_subregion_start);

        let node = self.nodes.get_mut(&id).unwrap();
        node.nv_subregion.pos = nv_subregion_start;
        if node.is_native {
            // native views take up exactly one space
            node.nv_subregion.len = 1;
            vec![id]
        } else {
            // all other views are composite views and take up as much space as their contents
            node.nv_subregion.len = subviews.len();
            subviews
        }
    }

    /// Adds a new view to the tree.
    fn add_view(
        &mut self,
        id: ViewId,
        view: &Arc<dyn View<Ctx>>,
        nv_subregion_start: usize,
        context: Ctx,
    ) {
        let is_native = view.native_type().is_some();
        let state = view.new_state(Context {
            // TODO: proper context
            context: context.clone(),
        });

        if is_native {
            self.patches
                .push_back(Patch::Update(id, view.native_view()));
        }

        self.nodes.insert(
            id,
            TreeNode {
                view: Arc::clone(view),
                is_native,
                superview: None,
                nv_ancestor: None,
                nv_subregion: Subregion {
                    pos: nv_subregion_start,
                    len: 0,
                },
                state,
                subviews: Vec::new(),
                context,
            },
        );
    }

    /// Removes a view and its subviews.
    ///
    /// Does *not* remove the view from the superview’s `subviews` list. The view must exist.
    fn remove_view(&mut self, id: ViewId, emit_patch: bool) {
        let node = self.nodes.remove(&id).expect("removing nonexistent view");
        if emit_patch && node.is_native {
            self.patches.push_back(Patch::Remove(id));
        }
        for subview in node.subviews {
            self.remove_view(subview, true);
        }
    }

    /// Replaces a view with another of a different type.
    ///
    /// The view must exist.
    fn replace_view(
        &mut self,
        id: ViewId,
        view: &Arc<dyn View<Ctx>>,
        nv_subregion_start: usize,
        context: Ctx,
    ) {
        let current = self.nodes.get(&id).expect("replacing nonexistent view");
        let superview = current.superview;
        let nv_ancestor = current.nv_ancestor;
        let was_native = current.is_native;
        let is_native = view.native_type().is_some();

        self.remove_view(id, false);
        self.add_view(id, view, nv_subregion_start, context);

        let node = self.nodes.get_mut(&id).unwrap();
        node.is_native = is_native;
        node.superview = superview;
        node.nv_ancestor = nv_ancestor;

        if was_native && is_native {
            self.patches
                .push_back(Patch::Replace(id, view.native_view()));
        } else if was_native {
            self.patches.push_back(Patch::Remove(id));
        } else if is_native {
            self.patches
                .push_back(Patch::Update(id, view.native_view()));
        }
    }

    /// Updates an existing view with new properties, which must be of the same type.
    fn update_view(&mut self, id: ViewId, view: &Arc<dyn View<Ctx>>) {
        let node = self.nodes.get_mut(&id).expect("updating nonexistent view");
        debug_assert!(
            node.view.as_any().type_id() == view.as_any().type_id(),
            "update_view called with incorrect type"
        );
        node.state.will_update(&**view);
        if node.is_native {
            self.patches
                .push_back(Patch::Update(id, view.native_view()));
        }
        node.view = Arc::clone(view);
    }

    /// Diffs the subview/the subviews of a node and returns the NV ids.
    fn diff_subviews(
        &mut self,
        superview: ViewId,
        subview: Arc<dyn View<Ctx>>,
        nv_subregion_start: usize,
    ) -> Vec<ViewId> {
        let superview_node = &self.nodes[&superview];
        // the closest native ancestor for the subview is either
        let nv_ancestor = if superview_node.is_native {
            // the superview itself
            Some(superview)
        } else {
            // or the superview’s native ancestor
            superview_node.nv_ancestor
        };
        let nv_subregion = superview_node.nv_subregion;

        let subview_context = match superview_node
            .view
            .subview_context(&superview_node.state, &superview_node.context)
        {
            Some(ctx) => ctx,
            None => superview_node.context.clone(),
        };

        let mut single_subview_storage = Vec::with_capacity(1);
        let subviews = match subview.as_any().downcast_ref::<Fragment<Ctx>>() {
            Some(subviews) => subviews, // list of subviews
            None => match subview.as_any().downcast_ref::<()>() {
                Some(()) => &single_subview_storage, // no subviews at all
                None => {
                    // single subview
                    single_subview_storage.push(subview);
                    &single_subview_storage
                }
            },
        };

        // This will expand an array of subviews as the superview’s (only) subviews.
        // To identify which existing subview and newly rendered subview are meant to be the same,
        // each subview has a key.

        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        enum Key {
            /// A user-specified key.
            Key(u64),
            /// An automatically assigned key.
            AutoKey(u64),
        }

        // If a subview doesn’t have a user-specified key, it’ll be auto-keyed sequentially by
        // index ignoring user-keyed items, e.g.
        //
        // array     [A, B, C(key=1), D(key=2), E]
        // auto-key   0  1                      2

        let mut auto_key_counter = 0;
        let mut current_subviews_by_id = HashMap::new();
        for id in &self.nodes[&superview].subviews {
            let key = self.nodes[&id].view.key().map(Key::Key).unwrap_or_else(|| {
                let k = auto_key_counter;
                auto_key_counter += 1;
                Key::AutoKey(k)
            });
            current_subviews_by_id.insert(key, *id);
        }

        let mut auto_key_counter = 0;
        let mut new_subviews = Vec::new();
        let mut nv_subviews = Vec::new();
        let mut nv_subregion_cursor = nv_subregion_start;

        for view in subviews.iter().map(|view| Arc::clone(view)) {
            let key = view.key().map(Key::Key).unwrap_or_else(|| {
                let k = auto_key_counter;
                auto_key_counter += 1;
                Key::AutoKey(k)
            });

            if let Some(subview_id) = current_subviews_by_id.remove(&key) {
                // this new subview already has a corresponding old subview
                let mut nvs = self.diff(
                    subview_id,
                    &view,
                    nv_subregion_cursor,
                    subview_context.clone(),
                );
                nv_subregion_cursor += nvs.len();
                nv_subviews.append(&mut nvs);
                new_subviews.push(subview_id);
            } else {
                // no existing view with the same key, needs to be created
                let subview_id = ViewId::new();

                let mut nvs = self.diff(
                    subview_id,
                    &view,
                    nv_subregion_cursor,
                    subview_context.clone(),
                );
                nv_subregion_cursor += nvs.len();
                nv_subviews.append(&mut nvs);

                let subview_node = self.nodes.get_mut(&subview_id).unwrap();
                subview_node.superview = Some(superview);
                subview_node.nv_ancestor = nv_ancestor;
                new_subviews.push(subview_id);
            };
        }

        // unused subviews need to be removed
        for (_, id) in current_subviews_by_id {
            self.remove_view(id, true);
        }

        if let Some(nv_ancestor) = nv_ancestor {
            self.patches.push_back(Patch::SubviewRegion(
                nv_ancestor,
                nv_subregion.pos,
                nv_subregion.len,
                nv_subviews.clone(),
            ));
        }

        let superview_node = self.nodes.get_mut(&superview).unwrap();
        superview_node.subviews = new_subviews;
        superview_node.nv_subregion.pos = nv_subregion_start;
        superview_node.nv_subregion.len = nv_subviews.len();
        nv_subviews
    }
}
