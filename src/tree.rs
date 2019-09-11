use crate::context::Context;
use crate::events::{EventHandler, EventType, EventTypeId, Hover, Key, Pointer, Scroll};
use crate::layer::Layer;
use crate::patch::{LayerPatch, Patch};
use crate::view::{Fragment, State, View, ViewId};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;

/// A tree of views.
#[derive(Debug)]
pub struct ViewTree {
    context: Context,
    root: ViewId,
    pending_root_render: bool,
    views: HashMap<ViewId, Arc<dyn View>>,
    event_handlers: EventHandlers,
    states: HashMap<ViewId, Box<dyn State>>,
    parents: HashMap<ViewId, ViewId>,
    subviews: HashMap<ViewId, Vec<ViewId>>,
    /// The closest native view ancestor for each view.
    native_ancestors: HashMap<ViewId, ViewId>,
}

impl ViewTree {
    pub fn new(root: Arc<dyn View>) -> ViewTree {
        let root_id = ViewId::new();

        ViewTree {
            context: Context {},
            root: root_id,
            pending_root_render: true,
            views: {
                let mut views = HashMap::new();
                views.insert(root_id, root);
                views
            },
            event_handlers: EventHandlers::new(),
            states: HashMap::new(),
            parents: HashMap::new(),
            subviews: HashMap::new(),
            native_ancestors: HashMap::new(),
        }
    }

    /// Updates the tree.
    pub fn update(&mut self) {
        // TODO: dispatch events

        let mut patches = Vec::new();

        if self.pending_root_render {
            self.pending_root_render = false;
            let view = self.views.remove(&self.root).expect("pending root render has no view?");
            self.diff_render(self.root, view, &mut patches);
        }
    }

    /// Adds a view.
    fn add_view(&mut self, id: ViewId, view: Arc<dyn View>) {
        let state = view.new_state();
        self.states.insert(id, state);
        self.subviews.insert(id, Vec::new());
        self.views.insert(id, view);
    }

    /// Removes a view. The view must exist.
    ///
    /// - `replacing_view`: if true, will not remove its parent relationship
    fn remove_view(&mut self, id: ViewId, replacing_view: bool, patches: &mut Vec<Patch>) {
        let state = self.states.get(&id).unwrap();
        state.will_disappear();
        let view = self.views.remove(&id).unwrap();
        self.states.remove(&id);
        self.event_handlers.remove_view(id);
        if !replacing_view {
            if let Some(parent) = self.parents.get(&id) {
                // parent subviews may not exist if this is a recursive call
                if let Some(subviews) = self.subviews.get_mut(parent) {
                    let pos = subviews.iter().position(|i| *i == id).unwrap();
                    subviews.remove(pos);
                }
            }
            self.parents.remove(&id);
            self.native_ancestors.remove(&id);

            if view.native_type().is_some() {
                // native views need to be removed from their native parent
                patches.push(Patch::remove(id));
            }
        }

        // also remove all subviews
        for id in self.subviews.remove(&id).unwrap() {
            self.remove_view(id, false, patches);
        }
    }

    /// Either creates, replaces, or updates a view.
    ///
    /// Relationships must have been set up *before* calling this method.
    fn diff_render(&mut self, id: ViewId, view: Arc<dyn View>, patches: &mut Vec<Patch>) {
        if let Some(current) = self.views.get(&id) {
            if current.type_id() == view.type_id() {
                // same kind of view; only need to diff props
                if !current.eq(&*view) {
                    // needs update
                    let state = self.states.get(&id).unwrap();
                    state.will_update(&*view);
                    self.views.insert(id, view);
                }
            } else {
                // different view; needs replacing
                self.remove_view(id, true, patches);
                self.add_view(id, view);

                let state = self.states.get(&id).unwrap();
                state.will_appear(&self.context);
            }
        } else {
            self.add_view(id, view);
        }

        let view = self.views.get(&id).unwrap();
        let state = self.states.get(&id).unwrap();

        if let Some(layer) = view.as_any().downcast_ref::<Layer>() {
            patches.push(Patch::update(
                id,
                LayerPatch::new(layer, id, &mut self.event_handlers),
            ));
        } else if let Some(()) = view.as_any().downcast_ref::<()>() {
            // don’t do anything else
            return;
        }

        let body = view.body(state.as_any());
        self.diff_render_subview(id, body.into(), patches);
    }

    fn diff_render_subview(
        &mut self,
        id: ViewId,
        subview: Arc<dyn View>,
        patches: &mut Vec<Patch>,
    ) {
        let parent_is_native = self.views.get(&id).unwrap().native_type().is_some();
        let mut is_fake_native_ancestor = false;
        let native_ancestor = if parent_is_native { Some(id) } else { None };

        if let Some(views) = subview.as_any().downcast_ref::<Fragment>() {
            if is_fake_native_ancestor {
                panic!("multiple subviews not allowed without any native ancestors");
            }

            // expand multiple subviews

            // all subviews that don’t have a key will be auto-keyed sequentially
            // this allows for setups like [fixed, fixed, variable number with keys, fixed]
            // where all fixed views will always be identified correctly
            let mut auto_key_counter = 0;

            #[derive(Clone, Copy, PartialEq, Eq, Hash)]
            enum Key {
                Key(u64),
                AutoKey(u64),
            }

            // current subviews: key -> id
            let mut current_views = HashMap::new();

            for id in self.subviews[&id].iter() {
                let key = self.views[id].key().map(Key::Key).unwrap_or_else(|| {
                    let k = auto_key_counter;
                    auto_key_counter += 1;
                    Key::AutoKey(k)
                });
                current_views.insert(key, *id);
            }

            auto_key_counter = 0;

            let mut new_subviews = Vec::new();

            for view in views.iter().map(|view| Arc::clone(view)) {
                let key = view.key().map(Key::Key).unwrap_or_else(|| {
                    let k = auto_key_counter;
                    auto_key_counter += 1;
                    Key::AutoKey(k)
                });

                let id = if let Some(id) = current_views.remove(&key) {
                    id
                } else {
                    // no existing view with the same key, needs to be created
                    let s_id = ViewId::new();
                    self.parents.insert(s_id, id);
                    if let Some(native_ancestor) = native_ancestor {
                        self.native_ancestors.insert(s_id, native_ancestor);
                        patches.push(Patch::subview(native_ancestor, s_id));
                    }
                    s_id
                };

                self.diff_render(id, view, patches);

                new_subviews.push(id);
            }

            // unused subviews need to be removed
            for (_, id) in current_views {
                self.remove_view(id, false, patches);
            }

            let mut order = Vec::with_capacity(new_subviews.len());
            self.subviews.insert(id, new_subviews);
        } else {
            // one subview
            let subviews = self.subviews.get_mut(&id).unwrap();

            if subviews.len() > 1 {
                // too many subviews

                if let Some(key) = subview.key() {
                    // if the subview has a key, keep the subview that matches

                    let mut to_remove = Vec::with_capacity(subviews.len());
                    let mut found_one_to_keep = false;
                    for id in subviews {
                        if self.views[id].key().map_or(true, |k| k != key) || found_one_to_keep {
                            to_remove.push(*id);
                        } else {
                            found_one_to_keep = true;
                        }
                    }

                    for id in to_remove {
                        self.remove_view(id, false, patches);
                    }
                } else {
                    // otherwise just pick the first one to be the one that gets diffed
                    let to_remove = subviews.drain(1..).collect::<Vec<_>>();
                    for id in to_remove {
                        self.remove_view(id, false, patches);
                    }
                }
            }

            let subviews = self.subviews.get_mut(&id).unwrap();

            // use id of the last remaining subview or create a new one
            let subview_id = if let Some(id) = subviews.get(0) {
                *id
            } else {
                let s_id = ViewId::new();
                // set up relationship with parent
                subviews.push(s_id);
                self.parents.insert(s_id, id);
                if let Some(native_ancestor) = native_ancestor {
                    self.native_ancestors.insert(s_id, native_ancestor);
                    patches.push(Patch::subview(native_ancestor, s_id));
                }
                s_id
            };

            self.diff_render(subview_id, subview, patches);
        }
    }

    /// Collects all native descendants of a view, in-order.
    fn collect_native_descendants(&self, id: ViewId, descendants: &mut Vec<ViewId>) {
        let subviews = self.subviews.get(&id).unwrap();
        for id in subviews {
            if self.views.get(&id).unwrap().native_type().is_some() {
                descendants.push(*id);
            } else {
                self.collect_native_descendants(*id, descendants);
            }
        }
    }

    pub fn enqueue_event<T: EventType>(&mut self, view: ViewId, event: T) {
        unimplemented!("dispatch event")
    }
}

/// Refers to a single event handler.
pub(crate) type HandlerId = (ViewId, EventTypeId);

/// Helper enum for EventHandlers.
#[derive(Debug)]
pub(crate) enum PolyEventHandler {
    Hover(EventHandler<Hover>),
    Pointer(EventHandler<Pointer>),
    Key(EventHandler<Key>),
    Scroll(EventHandler<Scroll>),
}

/// Helper trait for EventHandlers.
pub(crate) trait PolyEventHandlerType {
    fn type_id() -> EventTypeId;
    fn into(self) -> PolyEventHandler;
}

macro_rules! impl_peht {
    ($($t:tt),+) => {
        $(
            impl PolyEventHandlerType for EventHandler<$t> {
                fn type_id() -> EventTypeId {
                    $t::type_id()
                }
                fn into(self) -> PolyEventHandler {
                    PolyEventHandler::$t(self)
                }
            }
        )+
    }
}
impl_peht!(Hover, Pointer, Key, Scroll);

/// List of event handlers.
#[derive(Debug)]
pub(crate) struct EventHandlers {
    map: BTreeMap<HandlerId, PolyEventHandler>,
}

impl EventHandlers {
    fn new() -> EventHandlers {
        EventHandlers {
            map: BTreeMap::new(),
        }
    }

    pub(crate) fn add_handler<T: PolyEventHandlerType>(&mut self, view: ViewId, handler: T) {
        self.map.insert((view, T::type_id()), handler.into());
    }

    pub(crate) fn remove_handler(&mut self, view: ViewId, ty: EventTypeId) {
        self.map.remove(&(view, ty));
    }

    fn remove_view(&mut self, view: ViewId) {
        let keys_to_remove: Vec<_> = self
            .map
            .range((view, EventTypeId::MIN)..(view, EventTypeId::MAX))
            .map(|(k, _)| *k)
            .collect();
        for key in keys_to_remove {
            self.map.remove(&key);
        }
    }
}

#[test]
fn test_tree_diff_render() {
    use crate::impl_view;
    use std::any::Any;
    use std::sync::Mutex;

    // TODO: test native descendants/ancestors

    let root = ViewId::new();
    let mut patches = Vec::new();

    #[derive(Debug)]
    struct RootState;

    thread_local! {
        static STATE_UPDATE_COUNTER: Mutex<usize> = Mutex::new(0);
    }

    impl State for RootState {
        fn as_any(&self) -> &Any {
            self
        }

        fn will_update(&self, view: &dyn View) {
            let s = STATE_UPDATE_COUNTER.with(|s| {
                *s.lock().unwrap() += 1;
                *s.lock().unwrap()
            });
            let view = view
                .as_any()
                .downcast_ref::<RootView>()
                .expect("will_update did not get a RootView");
            assert_eq!(
                view.0, s,
                "will_update should be called with the RootView(n) update"
            );
        }
    }

    #[derive(Debug, PartialEq)]
    struct RootView(usize);
    impl_view! {
        RootView;
        fn new_state(&self) {
            Box::new(RootState)
        }
        fn body(&self, state: &RootState) {
            match self.0 {
                0 => Arc::new(Subview1),
                1 => Arc::new(()),
                2 => {
                    let sv1: Arc<dyn View> = Arc::new(Subview1);
                    Arc::new(Layer {
                        subviews: vec![sv1, Arc::new(())],
                        ..Layer::default()
                    })
                },
                _ => panic!(),
            }
        }
    }

    #[derive(Debug, PartialEq)]
    struct Subview1;
    impl_view! {
        Subview1;
        fn new_state(&self) {
            Box::new(())
        }
        fn body(&self, state: &()) {
            Arc::new(())
        }
    }

    let root_view = Arc::new(RootView(0));
    let mut tree = ViewTree::new(root_view.clone());
    tree.views.remove(&root);
    tree.diff_render(root, root_view, &mut patches);
    println!("{:#?}", tree);

    assert_eq!(
        tree.views.len(),
        3,
        "there should be three views: RootView, Subview1, ()"
    );
    assert_eq!(tree.parents.get(&root), None, "root view has no parent");
    let subview_id = tree.subviews[&root]
        .get(0)
        .expect("root view should have a subview");
    assert_eq!(
        tree.parents.get(&subview_id),
        Some(&root),
        "subview’s parent should be root"
    );
    assert_eq!(
        tree.subviews[&subview_id].len(),
        1,
        "subview should have one subview"
    );

    println!("applying new render");
    let root_view = Arc::new(RootView(1));
    tree.diff_render(root, root_view, &mut patches);
    println!("{:#?}", tree);

    assert_eq!(
        tree.views.len(),
        2,
        "there should be two views: RootView, ()"
    );
    let subview_id = tree.subviews[&root]
        .get(0)
        .expect("root view should have a subview");
    assert_eq!(
        tree.parents.get(&subview_id),
        Some(&root),
        "subview’s parent should be root"
    );

    let state_updates = STATE_UPDATE_COUNTER.with(|s| *s.lock().unwrap());
    assert_eq!(
        state_updates, 1,
        "RootState::will_update should have been called once"
    );

    println!("applying new render");
    let root_view = Arc::new(RootView(2));
    tree.diff_render(root, root_view, &mut patches);
    println!("{:#?}", tree);

    assert_eq!(tree.views.len(), 5, "there should be five views");
    let layer = tree.subviews[&root]
        .get(0)
        .expect("root view should have a subview (layer)");
    let subview1 = tree.subviews[&layer]
        .get(0)
        .expect("layer should have a subview");
    let subview2 = tree.subviews[&layer]
        .get(1)
        .expect("layer should have two subviews");
    assert_eq!(
        tree.parents.get(&subview1),
        Some(layer),
        "subview1’s parent should be layer"
    );
    assert_eq!(
        tree.parents.get(&subview2),
        Some(layer),
        "subview2’s parent should be layer"
    );
    assert_eq!(
        tree.native_ancestors.get(&subview2),
        Some(layer),
        "subview2’s native ancestor should be layer"
    );
    assert_eq!(
        tree.views.get(&subview1).unwrap().as_any().type_id(),
        Subview1.type_id(),
        "subview1 should be of type Subview1"
    );
}
