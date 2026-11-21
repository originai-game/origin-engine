//! Definitions copied unchanged from nsh-core (6fa6368f; file and lines in
//! each comment) that the vendored input.rs names; see PATCHED.md.
#![allow(dead_code)]
use crate::{Point, Points, px};

// nsh-core/src/ops.rs:16,19
/// Identifier of a node.
pub type NodeId = u32;
/// The root node's id.
pub const ROOT_ID: NodeId = 1;

// nsh-core/src/props.rs:13
/// Insets of each side, in points.
pub type EdgeInsets = crate::Edges<Points>;

pub mod animation {
    // nsh-core/src/animation/ops.rs:2
    pub type NodeId = u32;
}

// nsh-core/src/ops.rs:118-279
macro_rules! events {
    ($($variant:ident = $prop:literal $(as $js:literal)?: $doc:literal,)*) => {
        /// An event a node can listen to.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(u8)]
        pub enum EventKind {
            $(#[doc = $doc] $variant,)*
        }

        impl EventKind {
            /// Every kind, in discriminant order (`ALL[k as usize] == k`).
            pub const ALL: &'static [EventKind] = &[$(EventKind::$variant,)*];

            /// Parses an event's protocol name such as `onPress` (what
            /// platforms send and log; [`EventKind::prop_name`]).
            #[inline]
            pub fn from_prop(name: &str) -> Option<EventKind> {
                match name {
                    $($prop => Some(EventKind::$variant),)*
                    _ => None,
                }
            }

            /// The protocol name of this event (platforms send and log it;
            /// apps name handlers with [`EventKind::handler_name`]).
            #[inline]
            pub const fn prop_name(self) -> &'static str {
                match self {
                    $(EventKind::$variant => $prop,)*
                }
            }

            /// The handler prop apps set for this event (`onChange` for
            /// [`EventKind::ChangeText`]); empty for the events only the
            /// runtime listens to (a dialog's dismissal, the root's back
            /// button), which no prop name sets.
            #[inline]
            pub const fn handler_name(self) -> &'static str {
                match self {
                    $(EventKind::$variant => events!(@js $prop $($js)?),)*
                }
            }

            /// Parses a handler prop name such as `onPress`
            /// ([`EventKind::handler_name`]).
            #[inline]
            pub fn from_handler(name: &str) -> Option<EventKind> {
                if name.is_empty() {
                    return None;
                }
                #[allow(unreachable_patterns)]
                match name {
                    $(events!(@js $prop $($js)?) => Some(EventKind::$variant),)*
                    _ => None,
                }
            }
        }
    };
    (@js $prop:literal) => { $prop };
    (@js $prop:literal $js:literal) => { $js };
}

// The order is part of the platform protocols (`encode.rs`, `Native.java`):
// append new kinds at the end.
events! {
    Press = "onPress": "A tap / click (payload: [`EventPayload::Press`] where the backend knows the pointer, else none).",
    LongPress = "onLongPress": "A press held for about half a second (payload as `Press`).",
    PanStart = "onPanStart": "A drag started (payload: [`EventPayload::Pan`]).",
    PanEnd = "onPanEnd": "A drag ended (payload: [`EventPayload::Pan`] with velocity).",
    Refresh = "onRefresh": "Pull-to-refresh was triggered on a scroll or list.",
    LoadMore = "onLoadMore": "A list was scrolled close to its end.",
    ScrollEnd = "onScrollEnd": "Scrolling came to rest (payload: [`EventPayload::Scroll`]).",
    Load = "onLoad": "An image loaded (payload: [`EventPayload::Image`]).",
    Error = "onError": "An image failed to load (payload: [`EventPayload::Error`]).",
    ChangeText = "onChangeText" as "onChange": "The user edited an input (payload: [`EventPayload::Text`]).",
    SubmitEditing = "onSubmitEditing" as "onSubmit": "The return key was pressed in an input (payload: [`EventPayload::Text`]).",
    Focus = "onFocus": "An input, or a view that takes presses, took the keyboard focus.",
    Blur = "onBlur": "An input, or a view that takes presses, lost the keyboard focus.",
    PressDown = "onPressIn" as "onPressDown": "A press went down on a pressable view (a finger, a pen or a mouse button; payload as `Press`).",
    PressUp = "onPressOut" as "onPressUp": "The press ended: released, moved away or cancelled (payload as `Press`).",
    Layout = "onLayout": "The frame changed (payload: [`EventPayload::Layout`], relative to the parent).",
    PinchStart = "onPinchStart": "A pinch started (payload: `scale`, `velocity`).",
    PinchEnd = "onPinchEnd": "A pinch ended (payload: `scale`, `velocity`).",
    RotationStart = "onRotationStart": "A two-finger rotation started (payload: `rotation`, `velocity`).",
    RotationEnd = "onRotationEnd": "A two-finger rotation ended (payload: `rotation`, `velocity`).",
    AccessibilityAction = "onAccessibilityAction": "An assistive technology performed an action (payload: `actionName`).",
    Dismiss = "onDismiss" as "": "A screen was popped or a modal dismissed by the user or the system (`NavPage`'s `onDismiss`, `Dialog`'s `onClose`).",
    RequestClose = "onRequestClose" as "": "The user tried to close a modal natively (Escape or the back button, a sheet swiped down; payload: `reason`: `\"cancel\"` or `\"swipe\"`, `\"cancel\"` when none); `Dialog`'s `onCancel` and `onOpenChange` (a press on its backdrop part is handled in JS).",
    Show = "onShow" as "": "A modal finished appearing (`Dialog`'s `onOpen`).",
    Insets = "onInsets": "The insets of a screen or modal changed (payload: `top`, `right`, `bottom`, `left`).",
    LoadStart = "onLoadStart": "A web view, video or image started loading.",
    Message = "onMessage": "A web page posted a message (payload: `data`).",
    NavigationStateChange = "onNavigationStateChange": "A web view navigated (payload: `url`, `title`, `loading`, `canGoBack`, `canGoForward`).",
    Progress = "onProgress": "Video playback progressed (payload: `currentTime`, `duration`).",
    End = "onEnd": "A video played to its end.",
    Buffer = "onBuffer": "A video started or stopped buffering (payload: `isBuffering`).",
    ViewableItemsChange = "onViewableItemsChange": "The range of visible list items changed (payload: `first`, `last`).",
    BackPress = "onBackPress" as "": "The Android back button was pressed and nothing native handled it (root only).",
    Scroll = "onScroll": "A scroll view moved, at most once per frame (payload: `x`, `y`, `contentWidth`, `contentHeight`, `width`, `height`).",
    ScrollDragStart = "onScrollDragStart": "The user started dragging a scroll view (payload: [`EventPayload::Scroll`]).",
    ScrollDragEnd = "onScrollDragEnd": "The user let go of a scroll view (payload: [`EventPayload::Scroll`]).",
    MomentumStart = "onMomentumStart": "A scroll view started gliding after a fling (payload: [`EventPayload::Scroll`]).",
    MomentumEnd = "onMomentumEnd": "A scroll view stopped gliding (payload: [`EventPayload::Scroll`]).",
    Dialog = "onDialog": "An alert or action sheet closed (root only; payload: `id`, `index`, -1 when dismissed, and `value`, the text of its text field when it has one).",
    Touch = "onTouch": "Touches on the view or inside it, for the JS touch and responder handlers (payload: [`EventPayload::Touch`]).",
    SelectionChange = "onSelectionChange": "The selection (or caret) of an input moved (payload: `start`, `end` in UTF-16 units), or the selection of a selectable view changed (payload: `text`, the selected text as a copy gives it, empty once nothing is selected; `start`, where it starts in the view's whole text; `bounds`, the box around it in the view, to which the core adds `pageBounds`).",
    KeyPress = "onKeyPress": "A key was pressed in an input (payload: `key`: the character, `Backspace` or `Enter`).",
    LoadEnd = "onLoadEnd": "An image finished loading, or failed.",
    ImageInfo = "onImageInfo": "An `Image.getSize` / `Image.prefetch` request finished (root only; payload: `id`, `width` and `height` in pixels, or `error`).",
    TextLayout = "onTextLayout": "A text block was laid out (payload: [`EventPayload::TextLayout`]).",
    ContentSizeChange = "onContentSizeChange": "The size of an input's text changed (payload: `width`, `height`; raised by the core).",
    BackRequest = "onBackRequest" as "onBackPress": "The user pressed back on a screen (the header's back button, Android back, the desktops' back keys and mouse button, the browser's back), whether or not `popOnBack` let it pop.",
    NativeEvent = "onNativeEvent": "A registered native component emitted an event (payload: [`EventPayload::Native`]).",
    Clipboard = "onClipboard": "A clipboard read finished (root only; payload: `id`, `has`: whether it holds what was asked; for text `text`, unless only `has` was asked; for an image `uri` (a `data:` URI), `width` and `height`, unless only `has` was asked, or `tooLarge` (its size in bytes) for one over `platform::clipboard::MAX_IMAGE_BYTES`; for files `files`, their `file:` URLs one per line).",
    KeyCommand = "onKeyCommand": "A key matched a binding of `keymap` (`nsh`) (root only; payload: the binding's id, [`crate::keymap::KeyBinding::id`]).",
    Menu = "onMenu": "A menu item was chosen: the application menu's or a context menu's (root only; desktops; payload: `id`), or a context menu closed (`closed`: true, after its item).",
    MouseEnter = "onMouseEnter": "The mouse came over the view or a view inside it (never on touches; payload: [`EventPayload::Mouse`]).",
    MouseLeave = "onMouseLeave": "The mouse left the view and the views inside it, or the window (payload: [`EventPayload::Mouse`], where it was last).",
    FileDragEnter = "onFileDragEnter": "A local file drag entered an eligible drop destination, on desktops and iOS (payload: [`EventPayload::FileDrop`]).",
    FileDragLeave = "onFileDragLeave": "A local file drag left its drop destination, on desktops and iOS.",
    FileDrop = "onFileDrop": "Local files were dropped (payload: [`EventPayload::FileDrop`]).",
    MouseMove = "onMouseMove": "The mouse moved over the view or a view inside it, with or without a button down, or a modifier key changed while it rests there with none down (payload: [`EventPayload::Mouse`]; at most one per frame).",
    ContextMenu = "onContextMenu": "The secondary button was pressed over the view (payload: [`EventPayload::ContextMenu`]): the nearest listener around the view under the pointer hears it, unless a text input or selectable text is nearer.",
    CompositionChange = "onCompositionChange": "An input method's composition in an input started, changed or ended (payload: `composing`, `text`: the text being composed, empty once it ends; after the `onChangeText` of the committed text).",
    PressOutside = "onPressOutside": "A press went down outside an overlay, its anchor and the overlays inside it (payload: `pageX`, `pageY`; raised by the core).",
    Cancel = "onCancel": "Escape or the Android back button was pressed while the overlay was the top one listening to it (raised by the core).",
    Position = "onPosition": "The core placed an overlay (payload: `placement`, the side it went on, `x`, `y`, `width`, `height` in the root view, `availableWidth`, `availableHeight`: the room on that side).",
    Host = "onHost": "The embedding host answered a [`Command::HostCall`] or sent an event to `host`'s listeners (`nsh`) (root only; payload: [`EventPayload::Json`], `{ type: \"reply\", id, ok, value?, error? }` or `{ type: \"event\", event, payload }`).",
    Pick = "onPick": "A picker closed (root only; payload: [`EventPayload::Picked`], no files when cancelled).",
    LoadMoreAtStart = "onLoadMoreAtStart": "A list was scrolled close to its start after the user or a scroll command moved it (raised by the core).",
    OpenUrl = "onOpenUrl": "A [`Command::OpenUrl`] request finished (root only; payload: `id`, `opened`: whether the platform opened the URL).",
    CaretMove = "onCaretMove": "An input's caret moved: with its selection, or as its text scrolled or wrapped again (payload: `bounds`, the caret in the input; the core adds `pageBounds`, in the root view).",
    Paste = "onPaste": "The user pasted into an input (payload: [`EventPayload::Paste`]: the text, files with their bytes such as a screenshot, local files' URLs), before the `onChangeText` of the text it inserts.",
    ScreenColor = "onScreenColor": "A [`Command::PickScreenColor`] request finished (root only; payload: `id`, `color`: the color picked, packed `0xRRGGBBAA`, absent when cancelled).",
    FileRequest = "onFileRequest": "A [`Command::FileRequest`] finished (root only; payload: [`EventPayload::FileRequest`]).",
    Speech = "onSpeech": "A speech recognition session reported (root only; payload: [`EventPayload::Speech`]).",
    Download = "onDownload": "A web view's download started, finished, failed or was cancelled (payload: `state`, `id`, `url`, `filename`, `path`, `source`; `bytes` once finished, `code` and `message` when it failed, `reason` when it was cancelled).",
    Notification = "onNotification": "The platform answered a [`Command::Notification`], or the user pressed a notification (root only; payload: the fields of [`crate::notifications::NotificationEvent`]).",
    UrlScheme = "onUrlScheme": "The system handed the app a URL that opens it (root only; payload: `url`, [`crate::url_scheme::URL`]); the core keeps those of the schemes the app claims ([`crate::url_scheme`]).",
    SecondInstance = "onSecondInstance": "A later launch of the app handed the running instance its command line (root only; desktops, nsh.json `singleInstance`; payload: [`EventPayload::Json`], [`crate::single_instance::SecondInstance::to_json`]).",
    Audio = "onAudio": "A sound or a player reported (root only; payload: [`EventPayload::Audio`]).",
    Window = "onWindow" as "": "A window of the app's own reported its state, a close press or that it closed (on its root node; desktops; payload: [`EventPayload::Window`]).",
    EditAction = "onEditAction" as "": "An action was chosen from the edit menu a [`Command::OpenEditMenu`] opened on an input (payload: `action`, an [`EditAction`]'s name); Paste is the input's own paste (`onPaste`), never reported.",
    Wheel = "onWheel": "A mouse wheel or touchpad scrolled over the view (payload: [`EventPayload::Wheel`]): the nearest listener around the view under the pointer hears it; one nearer than any scroll view under the pointer takes the wheel, so nothing scrolls or zooms natively ([`crate::input::wheel_listener`]).",
    Pinch = "onPinch": "A touchpad pinch over the view (payload: [`EventPayload::Pinch`]; macOS, Linux, GPUI, Safari): the nearest listener around the view under the pointer hears it; one nearer than any view that pinches natively takes it, so nothing zooms or pinches natively ([`crate::input::wheel_listener`]); a pinch stays on the view its start went to.",

    WebEvaluation = "onWebEvaluation" as "": "A native WebView evaluation reply (internal; payload: id and a JSON envelope string data).",
    MouseUp = "onMouseUp": "A mouse button was released over the view or a view inside it (payload: [`EventPayload::Mouse`], its `button` naming the released one, with where and the modifiers). The primary button's release also reports `onPressUp`.",
    CanvasImage = "onCanvasImage": "A [`Command::RenderCanvas`] request finished (root only; payload: [`EventPayload::CanvasImage`]).",
    KeyDown = "onKeyDown": "A hardware key went down on the focused view or a view inside it (payload: [`EventPayload::Key`]). Notifications bubble to enabled listening ancestors within the same native window; returning from the listener cannot change handling, and `default_prevented` reports the mounted capture policy's decision for this transition.",
    KeyUp = "onKeyUp": "A hardware key came up on the focused view (payload: [`EventPayload::Key`]), with `default_prevented` as the capture policy decided for this transition. It bubbles to enabled ancestors as `onKeyDown` does.",
    WebViewReply = "onWebViewReply" as "": "A [`Command::WebViewRequest`] was carried out (the web view it was sent to; payload: [`EventPayload::Json`], `{ id, ok, value }` or `{ id, ok, error }`; the core answers the request's typed result and it reaches no listener).",
    DownloadPending = "onDownloadPending" as "": "A web view's page asked to download a file and waits for the app's destination (payload: [`EventPayload::Json`]; handed to Rust's `WebView::on_download_pending`, not to a JS listener).",
    DownloadOffer = "onDownloadOffer" as "": "A platform holds a download for the core (payload: [`EventPayload::Json`], [`crate::webview::DownloadOffer`]; never reaches a listener).",
    DownloadProgress = "onDownloadProgress" as "": "A held download received bytes (payload: [`EventPayload::Json`], [`crate::webview::DownloadProgress`]; never reaches a listener).",
    DownloadEnd = "onDownloadEnd" as "": "A platform's download ended (payload: [`EventPayload::Json`], [`crate::webview::DownloadEnd`]; never reaches a listener).",
    Share = "onShare": "A [`Command::Share`] request finished (root only; payload: `id`, then `action` (`\"shared\"` with the `target` where the platform names it, or `\"dismissed\"`), or `error` with its `code`; [`crate::share`]).",
    DetentChange = "onDetentChange": "The user moved an iOS sheet dialog to another of its `detents` (dragging it, its grabber, a scroll at its edge, a screen reader; payload: [`EventPayload::Json`], `{ detent }`, [`SheetDetent::change_payload`]).",
    AuthSession = "onAuthSession" as "": "A [`Command::AuthSession`] request finished (root only; payload: `id`, then the callback `url`, `action` `\"cancelled\"`, or `error` with its `code`; [`crate::auth_session`]; never reaches a listener).",
    FocusIn = "onFocusIn": "The keyboard focus came into the view: onto it or a view inside it, from outside (it entered `focus-within`; raised by the core). Moving between views inside it is no new coming in.",
    FocusOut = "onFocusOut": "The keyboard focus left the view: from it or a view inside it to one outside, or to none (it left `focus-within`; raised by the core).",
}

// nsh-core/src/ops.rs:982-1030
// Every kind has a bit of [`EventMask`].
const _: () = assert!(EventKind::ALL.len() <= 128, "EventMask has 128 bits");

/// A set of [`EventKind`]s: one bit per kind, room for 128. Two words, not
/// a `u128`: views keep it in Objective-C ivars, which take no 16-byte
/// alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EventMask([u64; 2]);

impl std::ops::BitOr for EventMask {
    type Output = EventMask;

    #[inline]
    fn bitor(self, other: EventMask) -> EventMask {
        EventMask([self.0[0] | other.0[0], self.0[1] | other.0[1]])
    }
}

impl EventMask {
    /// Returns true if `event` is in the set.
    #[inline]
    pub const fn contains(self, event: EventKind) -> bool {
        let k = event as usize;
        self.0[k / 64] & (1 << (k % 64)) != 0
    }

    /// The raw bit set, indexed by `EventKind as u8`.
    #[inline]
    pub const fn bits(self) -> u128 {
        self.0[0] as u128 | (self.0[1] as u128) << 64
    }

    /// Adds or removes `event`.
    #[inline]
    pub fn set(&mut self, event: EventKind, enabled: bool) {
        let k = event as usize;
        if enabled {
            self.0[k / 64] |= 1 << (k % 64);
        } else {
            self.0[k / 64] &= !(1 << (k % 64));
        }
    }

    /// Returns true if no event is set.
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0[0] == 0 && self.0[1] == 0
    }
}

// nsh-core/src/props.rs:585-683
/// When a pan on a view activates (`panActivation`; react-native-gesture-
/// handler's `activeOffsetX` / `failOffsetY`), decided by the platform on
/// the main thread as the fingers move: once the finger has moved further
/// than an active offset from where it went down, the pan activates (the
/// view becomes the touch responder and the scroll views around it can no
/// longer take these touches); once it has moved further than a fail offset
/// first, the pan fails and the scroll views keep the touches. Offsets are
/// distances in points along one axis, either direction. Without active
/// offsets, the pan activates after [`PanActivation::MIN_DISTANCE`] in any
/// direction.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PanActivation {
    /// Activates once the finger moved further than this horizontally.
    pub active_offset_x: Option<Points>,
    /// Activates once the finger moved further than this vertically.
    pub active_offset_y: Option<Points>,
    /// Fails once the finger moved further than this horizontally first.
    pub fail_offset_x: Option<Points>,
    /// Fails once the finger moved further than this vertically first.
    pub fail_offset_y: Option<Points>,
}

impl PanActivation {
    /// How far the finger moves before a pan without active offsets
    /// activates, in points.
    pub const MIN_DISTANCE: Points = px(10.0);

    /// A pan that activates horizontally past `active` and fails when it
    /// moves vertically past `fail` first (a scrubber in a vertical scroll
    /// view).
    pub const fn horizontal(active: Points, fail: Points) -> Self {
        Self {
            active_offset_x: Some(active),
            active_offset_y: None,
            fail_offset_x: None,
            fail_offset_y: Some(fail),
        }
    }

    /// A pan that activates vertically past `active` and fails when it
    /// moves horizontally past `fail` first.
    pub const fn vertical(active: Points, fail: Points) -> Self {
        Self {
            active_offset_x: None,
            active_offset_y: Some(active),
            fail_offset_x: Some(fail),
            fail_offset_y: None,
        }
    }

    /// The one axis this pan is for, when its offsets name one: `Some(true)`
    /// horizontal (an active offset along x, or a fail offset along y),
    /// `Some(false)` vertical, `None` either. A view listening to `onWheel`
    /// with a pan along one axis takes only the wheels along it
    /// ([`crate::input::wheel_along`]).
    pub fn axis(&self) -> Option<bool> {
        let x = self.active_offset_x.is_some() || self.fail_offset_y.is_some();
        let y = self.active_offset_y.is_some() || self.fail_offset_x.is_some();
        match (x, y) {
            (true, false) => Some(true),
            (false, true) => Some(false),
            _ => None,
        }
    }

    /// The decision once the finger moved by (`dx`, `dy`) since it went
    /// down: `Some(true)` activates, `Some(false)` fails (a fail offset
    /// passed wins over an active one passed by the same move), `None`
    /// waits. The platforms ask it on every move until it decides.
    pub fn decide(&self, dx: Points, dy: Points) -> Option<bool> {
        let (dx, dy) = (dx.0, dy.0);
        let past = |limit: Option<Points>, d: f32| limit.is_some_and(|l| d.abs() > l.0);
        if past(self.fail_offset_x, dx) || past(self.fail_offset_y, dy) {
            return Some(false);
        }
        let activates = if self.active_offset_x.is_some() || self.active_offset_y.is_some() {
            past(self.active_offset_x, dx) || past(self.active_offset_y, dy)
        } else {
            dx.hypot(dy) > Self::MIN_DISTANCE.0
        };
        activates.then_some(true)
    }

    /// [`PanActivation::decide`] on a move of the finger from offset `from`
    /// to offset `to` (both from where it went down), judged along the
    /// straight line between them a point at a time: the offset the finger
    /// passed first decides, even when the move passes both. Moves arrive
    /// several points apart, and a diagonal 1.5 across for 1 down passes
    /// `activeOffsetX: 10` and `failOffsetY: 10` within one 10-point move.
    /// The platforms ask it with the previous move's offset as `from`.
    pub fn decide_move(&self, from: Point<Points>, to: Point<Points>) -> Option<bool> {
        let (mx, my) = (to.x.0 - from.x.0, to.y.0 - from.y.0);
        let steps = mx.hypot(my).ceil().max(1.0) as u32;
        (1..=steps).find_map(|i| {
            let t = i as f32 / steps as f32;
            self.decide(Points(from.x.0 + mx * t), Points(from.y.0 + my * t))
        })
    }
}

// nsh-core/src/props.rs:686-698
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GestureValues {
    /// Horizontal drag translation (`pan={{ x }}`).
    pub pan_x: Option<crate::animation::NodeId>,
    /// Vertical drag translation (`pan={{ y }}`).
    pub pan_y: Option<crate::animation::NodeId>,
    /// Pinch scale (`pinch={{ scale }}`), multiplied from its value when the
    /// pinch starts.
    pub pinch: Option<crate::animation::NodeId>,
    /// Two-finger rotation in degrees (`rotation={{ angle }}`), added to its
    /// value when the rotation starts.
    pub rotation: Option<crate::animation::NodeId>,
}

// nsh-core/src/mount.rs:19-58
/// Kind of native view to create.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewKind {
    /// A plain container view.
    View,
    /// A text label.
    Text,
    /// An image.
    Image,
    /// A scroll view (also used by lists).
    Scroll,
    /// A text input.
    Input,
    /// A navigation stack: its children are screens, pushed and popped
    /// natively.
    Stack,
    /// A screen of a stack (its root view; the platform also makes its
    /// header).
    Screen,
    /// The root view of a modal: inserting it presents the modal, removing
    /// it dismisses it.
    Modal,
    /// A view shown in a screen's header (its slot is in the screen's
    /// [`ScreenConfig`]).
    HeaderItem,
    /// A bar over the keyboard ([`InputConfig::accessory`]); its frame's
    /// size is its own, the platform places it.
    InputAccessory,
    /// A canvas ([`MountItem::Canvas`]).
    Canvas,
    /// A web view.
    WebView,
    /// The container of a registered native component: the platform
    /// creates the component in it ([`MountItem::Component`]) and fills it
    /// with the component's view; decoration and frames are the
    /// container's.
    Native,
    /// A video player.
    Video,
}

pub mod keymap {
    // nsh-core/src/keymap.rs:25-74
/// The modifiers of a keystroke.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Modifiers {
    /// Control.
    pub ctrl: bool,
    /// Option on Apple platforms, Alt elsewhere.
    pub alt: bool,
    /// Shift.
    pub shift: bool,
    /// Command on Apple platforms; the Super (Windows) key elsewhere.
    pub cmd: bool,
}

/// A key, by what it is on the US layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// A key typing a printable ASCII character without Shift: a lower
    /// case letter, a digit, punctuation, or `' '` (space).
    Char(char),
    /// Return / Enter (the keypad's too).
    Enter,
    /// Escape.
    Escape,
    /// Tab.
    Tab,
    /// Backspace (the Mac's Delete).
    Backspace,
    /// Forward delete.
    Delete,
    /// Insert (Help on Apple keyboards).
    Insert,
    /// Up arrow.
    Up,
    /// Down arrow.
    Down,
    /// Left arrow.
    Left,
    /// Right arrow.
    Right,
    /// Home.
    Home,
    /// End.
    End,
    /// Page Up.
    PageUp,
    /// Page Down.
    PageDown,
    /// A function key, F1 to F24.
    F(u8),
}

}
