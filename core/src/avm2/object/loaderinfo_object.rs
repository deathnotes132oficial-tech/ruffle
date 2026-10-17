//! Loader-info object

use crate::avm2::activation::Activation;
use crate::avm2::events;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{EventObject, FunctionObject, Object, StageObject, TObject};
use crate::avm2::{Avm2, Error};
use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, TDisplayObject, TDisplayObjectContainer};
use crate::loader::{ContentType, LoaderHandle};
use crate::string::AvmString;
use crate::tag_utils::{SwfMovie, SwfMovieData};
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::{Collect, Gc, GcWeak, Mutation, lock::Lock};
use ruffle_common::utils::HasPrefixField;
use ruffle_macros::istr;
use std::cell::Cell;

/// Represents a thing which can be loaded by a loader.
#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub enum LoaderStream<'gc> {
    /// An SWF movie that has not yet loaded.
    ///
    /// The main differences from `Swf` loader streams is that certain loader
    /// info properties are `null` until the SWF is fully loaded. Furthermore,
    /// the `DisplayObject` parameter is optional, to represent movies that do
    /// not yet have a root clip.
    ///
    /// While the `Stage` is not a loadable object, it has `loaderInfo`, with
    /// properties that roughly mirror an unloaded movie clip. Properties that
    /// are valid on `Stage.loaderInfo` will be pulled from the root SWF.
    ///
    /// The `bool` parameter indicates if this is the `Stage`'s loader info;
    /// this is because certain `Stage` properties are accessible even when the
    /// associated movie is not yet loaded.
    NotYetLoaded(SwfMovie<'gc>, Option<DisplayObject<'gc>>, bool),

    /// A loaded SWF movie.
    ///
    /// The associated `DisplayObject` is the root movieclip.
    Swf(SwfMovie<'gc>, DisplayObject<'gc>),
}

impl<'gc> LoaderStream<'gc> {
    pub fn movie(&self) -> SwfMovie<'gc> {
        match self {
            LoaderStream::NotYetLoaded(movie, _, _) => *movie,
            LoaderStream::Swf(movie, _) => *movie,
        }
    }
}

/// An Object which represents a loadable object, such as a SWF movie or image
/// resource.
#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct LoaderInfoObject<'gc>(pub Gc<'gc, LoaderInfoObjectData<'gc>>);

#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub struct LoaderInfoObjectWeak<'gc>(pub GcWeak<'gc, LoaderInfoObjectData<'gc>>);

impl<'gc> LoaderInfoObjectWeak<'gc> {
    pub fn upgrade(self, mc: &Mutation<'gc>) -> Option<LoaderInfoObject<'gc>> {
        self.0.upgrade(mc).map(LoaderInfoObject)
    }
}

impl fmt::Debug for LoaderInfoObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoaderInfoObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, Clone, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct LoaderInfoObjectData<'gc> {
    /// All normal script data.
    base: ScriptObjectData<'gc>,

    /// The loaded stream that this gets its info from.
    loaded_stream: Lock<LoaderStream<'gc>>,

    loader: Option<StageObject<'gc>>,

    creator: Option<LoaderInfoObject<'gc>>,

    /// Whether or not we've fired our 'init' event
    init_event_fired: Cell<bool>,

    /// Whether or not we've fired our 'complete' event
    complete_event_fired: Cell<bool>,

    /// The `EventDispatcher` used for `LoaderInfo.sharedEvents`.
    // FIXME: If we ever implement sandboxing, then ensure that we allow
    // events to be fired across security boundaries using this object.
    shared_events: Object<'gc>,

    uncaught_error_events: Object<'gc>,

    content_type: Cell<ContentType>,

    expose_content: Cell<bool>,

    errored: Cell<bool>,
}

impl<'gc> LoaderInfoObject<'gc> {
    /// Create a loader info object that has not yet been loaded.
    ///
    /// Use `None` as the root clip to indicate that this is the stage's loader
    /// info.
    pub fn not_yet_loaded(
        activation: &mut Activation<'_, 'gc>,
        movie: SwfMovie<'gc>,
        loader: Option<StageObject<'gc>>,
        creator: Option<LoaderInfoObject<'gc>>,
        root_clip: Option<DisplayObject<'gc>>,
        is_stage: bool,
    ) -> Result<Self, Error<'gc>> {
        let class = activation.avm2().classes().loaderinfo;
        let base = ScriptObjectData::new(class);

        let object = LoaderInfoObject(Gc::new(
            activation.gc(),
            LoaderInfoObjectData {
                base,
                loaded_stream: Lock::new(LoaderStream::NotYetLoaded(movie, root_clip, is_stage)),
                loader,
                creator,
                init_event_fired: Cell::new(false),
                complete_event_fired: Cell::new(false),
                shared_events: activation
                    .context
                    .avm2
                    .classes()
                    .eventdispatcher
                    .construct(activation, &[])?
                    .as_object()
                    .unwrap(),
                uncaught_error_events: activation
                    .context
                    .avm2
                    .classes()
                    .uncaughterrorevents
                    .construct(activation, &[])?
                    .as_object()
                    .unwrap(),
                content_type: Cell::new(ContentType::Unknown),
                expose_content: Cell::new(false),
                errored: Cell::new(false),
            },
        ));

        Ok(object)
    }

    /// The `LoaderInfo` of `movie`'s code: the one its `Loader` loaded it
    /// into, or the root's for the root movie and for code whose `LoaderInfo`
    /// is gone.
    pub fn of_movie(context: &UpdateContext<'gc>, movie: SwfMovie<'gc>) -> Option<Self> {
        context
            .library
            .loader_info(context.gc(), movie)
            .or_else(|| context.stage.root_clip()?.loader_info())
    }

    pub fn downgrade(self) -> LoaderInfoObjectWeak<'gc> {
        LoaderInfoObjectWeak(Gc::downgrade(self.0))
    }

    pub fn loader(self) -> Option<StageObject<'gc>> {
        self.0.loader
    }

    pub fn creator(self) -> Option<LoaderInfoObject<'gc>> {
        self.0.creator
    }

    pub fn shared_events(self) -> Object<'gc> {
        self.0.shared_events
    }

    pub fn uncaught_error_events(self) -> Object<'gc> {
        self.0.uncaught_error_events
    }

    /// Gets the `ContentType`, 'hiding' it by returning `ContentType::Unknown`
    /// if we haven't yet fired the 'init' event. The real ContentType first becomes
    /// visible to ActionScript in the 'init' event.
    pub fn content_type_hide_before_init(self) -> ContentType {
        if self.0.init_event_fired.get() {
            self.0.content_type.get()
        } else {
            ContentType::Unknown
        }
    }

    pub fn set_errored(self, val: bool) {
        self.0.errored.set(val);
    }

    pub fn errored(self) -> bool {
        self.0.errored.get()
    }

    pub fn init_event_fired(self) -> bool {
        self.0.init_event_fired.get()
    }

    pub fn reset_init_and_complete_events(self) {
        self.0.init_event_fired.set(false);
        self.0.complete_event_fired.set(false);
    }

    /// Fires the 'init' and 'complete' events if they haven't been fired yet.
    /// Returns `true` if both events have been fired (either as a result of
    /// this call, or due to a previous call), or if a handler cancelled
    /// `load`, the load these events belong to (`None` for the root movie).
    pub fn fire_init_and_complete_events(
        self,
        context: &mut UpdateContext<'gc>,
        load: Option<LoaderHandle>,
        status: u16,
        redirected: bool,
    ) -> bool {
        let cancelled = |context: &UpdateContext<'gc>| {
            load.is_some_and(|load| context.load_manager.is_cancelled(load))
        };
        self.0.expose_content.set(true);
        if !self.0.init_event_fired.get() {
            self.0.init_event_fired.set(true);

            // TODO - 'init' should be fired earlier during the download.
            // Right now, we fire it when downloading is fully completed.
            let init_evt = EventObject::bare_default_event(context, "init");
            Avm2::dispatch_event(context, init_evt, self.into());
            if cancelled(context) {
                return true;
            }
        }

        if !self.0.complete_event_fired.get() {
            // NOTE: We have to check load progress here because this function
            // is called unconditionally at the end of every frame.
            let (should_complete, from_bytes) = match self.0.loaded_stream.get() {
                LoaderStream::Swf(movie, root) => (
                    root.as_movie_clip()
                        .map(|mc| mc.loaded_bytes() as i32 >= mc.total_bytes())
                        .unwrap_or(true),
                    movie.is_from_bytes(),
                ),
                _ => (false, false),
            };

            if should_complete {
                let mut activation = Activation::from_nothing(context);
                if !from_bytes {
                    let http_status_evt =
                        EventObject::http_status_event(&mut activation, status, redirected);

                    Avm2::dispatch_event(context, http_status_evt, self.into());
                    if cancelled(context) {
                        return true;
                    }
                }

                self.0.complete_event_fired.set(true);
                let complete_evt = EventObject::bare_default_event(context, "complete");
                Avm2::dispatch_event(context, complete_evt, self.into());
                return true;
            }
            return false;
        }
        true
    }

    /// Unwrap this object's loader stream
    pub fn loader_stream(self) -> LoaderStream<'gc> {
        self.0.loaded_stream.get()
    }

    pub fn expose_content(self) -> bool {
        self.0.expose_content.get()
    }

    /// Makes the 'content' visible to ActionScript.
    /// This is used by certain special loaders (the stage and root movie),
    /// which expose the loaded content before the 'init' event is fired.
    pub fn set_expose_content(self) {
        self.0.expose_content.set(true);
    }

    pub fn set_loader_stream(&self, stream: LoaderStream<'gc>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, self.0), LoaderInfoObjectData, loaded_stream).set(stream);
    }

    pub fn set_content_type(self, content_type: ContentType) {
        self.0.content_type.set(content_type);
    }

    /// Removes the Loader's content and cancels every load into this
    /// `LoaderInfo`, whatever stage it has reached.
    pub fn unload(self, context: &mut UpdateContext<'gc>) {
        context.load_manager.cancel_all_avm2_loads(self);

        let mut loader = self
            .0
            .loader
            .expect("LoaderInfo must have been created by Loader")
            .display_object()
            .as_container()
            .unwrap();

        let content = loader.child_by_index(0);

        if content.is_some() {
            let unload_evt = EventObject::bare_default_event(context, "unload");
            Avm2::dispatch_event(context, unload_evt, self.into());
        }

        // Reset properties
        let movie = &context.root_swf;
        let empty_swf = SwfMovie::new(
            context.gc(),
            SwfMovieData::empty(movie.version(), Some(movie.url().into())),
        );
        let loader_stream = LoaderStream::NotYetLoaded(empty_swf, None, false);
        self.set_loader_stream(loader_stream, context.gc());
        self.set_errored(false);
        self.reset_init_and_complete_events();

        // Remove the Loader's content element if it exists.
        if let Some(child) = content {
            loader.remove_child(context, child);
        }
    }

    /// Unloads like [`Self::unload`], first stopping what the content runs,
    /// as Flash documents for `Loader.unloadAndStop`: the timelines, sounds,
    /// and frame and activation listeners of its display objects, and the
    /// stage listeners, timers and loads that its code started. Every movie
    /// that its code loaded, at any depth, goes the same way.
    pub fn unload_and_stop(self, context: &mut UpdateContext<'gc>) {
        let stream = self.loader_stream();
        let content = match stream {
            LoaderStream::Swf(_, root) => Some(root),
            LoaderStream::NotYetLoaded(_, root, _) => root,
        };

        if let Some(content) = content {
            let loaded = context
                .library
                .movies_loaded_by(context.gc(), stream.movie());
            let contents: Vec<_> = std::iter::once(content)
                .chain(
                    loaded
                        .iter()
                        .filter_map(|movie| content_root(context, *movie)),
                )
                .collect();
            let unload_events = unload_events(context);

            for content in &contents {
                context.stop_sounds_on_parent_and_children(*content);
                stop_content(context, *content, &contents, &unload_events);
            }

            let movies: Vec<_> = std::iter::once(stream.movie()).chain(loaded).collect();
            debug_assert!(!movies.contains(context.root_swf));
            context.load_manager.cancel_loads_started_by(&movies);
            context.stop_sounds_started_by(&movies);
            let mc = context.gc();
            context.timers.remove_owned_by(mc, &movies);
            remove_stage_listeners_of(context, &movies);
            remove_unload_listeners_of(context, &movies, &unload_events);
        }

        self.unload(context);
    }
}

/// `render` is not in Flash's documented list; Artix's Ruffle build removes it
/// too.
fn unload_events<'gc>(context: &UpdateContext<'gc>) -> [AvmString<'gc>; 6] {
    [
        istr!(context, "enterFrame"),
        istr!(context, "exitFrame"),
        istr!(context, "frameConstructed"),
        istr!(context, "render"),
        istr!(context, "activate"),
        istr!(context, "deactivate"),
    ]
}

fn content_root<'gc>(
    context: &UpdateContext<'gc>,
    movie: SwfMovie<'gc>,
) -> Option<DisplayObject<'gc>> {
    context
        .library
        .library_for_movie(movie)?
        .content_root(context.gc())
}

fn stop_content<'gc>(
    context: &mut UpdateContext<'gc>,
    object: DisplayObject<'gc>,
    contents: &[DisplayObject<'gc>],
    unload_events: &[AvmString<'gc>],
) {
    if let Some(clip) = object.as_movie_clip() {
        clip.stop(context);
    }

    if let Some(object) = object.object2() {
        events::retain_listeners(context.gc(), object.into(), |event, _| {
            !unload_events.contains(&event)
        });
    }

    if let Some(container) = object.as_container() {
        for child in container.iter_render_list() {
            if !contents
                .iter()
                .any(|content| DisplayObject::ptr_eq(*content, child))
            {
                stop_content(context, child, contents, unload_events);
            }
        }
    }
}

fn is_owned_by<'gc>(movies: &[SwfMovie<'gc>], handler: FunctionObject<'gc>) -> bool {
    movies.contains(&handler.executable().as_method().owner_movie())
}

fn remove_stage_listeners_of<'gc>(context: &mut UpdateContext<'gc>, movies: &[SwfMovie<'gc>]) {
    if let Some(stage) = context.stage.object2() {
        events::retain_listeners(context.gc(), stage.into(), |_, handler| {
            !is_owned_by(movies, handler)
        });
    }
}

fn remove_unload_listeners_of<'gc>(
    context: &mut UpdateContext<'gc>,
    movies: &[SwfMovie<'gc>],
    unload_events: &[AvmString<'gc>],
) {
    for object in context.avm2.broadcast_listeners(context.gc()) {
        events::retain_listeners(context.gc(), object, |event, handler| {
            !(unload_events.contains(&event) && is_owned_by(movies, handler))
        });
    }
}

impl<'gc> TObject<'gc> for LoaderInfoObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}
