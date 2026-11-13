//! How a document writes each kind: its tags, HTML's name wherever HTML has
//! the role (docs/ui.md §2), each with the example the catalogue shows, and
//! the `og-` tags for engine semantics HTML has no name for.

use super::Kind;

/// Author tags for engine semantics without an HTML equivalent, mapped
/// to the runtime kinds used by checkpoints.
pub const ELEMENT_TAGS: &[(&str, Kind)] = &[
    ("og-icon", Kind::Icon),
    ("og-counter", Kind::Label),
    ("og-typewriter", Kind::Label),
    ("og-spotlight", Kind::Dialog),
    ("og-rive", Kind::Column),
    ("og-emitter", Kind::Box),
    ("og-overlay", Kind::Box),
];

/// Tag-exclusive attributes and the runtime properties they populate.
pub const ELEMENT_ATTRIBUTES: &[(&str, &str, &str)] = &[
    ("og-counter", "value", "count"),
    ("og-counter", "duration", "countDuration"),
    ("og-counter", "decimals", "countDecimals"),
    ("og-typewriter", "rate", "reveal"),
    ("og-spotlight", "target", "spotlight"),
    ("og-rive", "src", "rive"),
    ("og-emitter", "effect", "emitter"),
    ("og-overlay", "effect", "meta"),
];

/// The tags a document writes `kind` as, each with an example. Every kind
/// is named, so a new one says how it is written.
pub fn of(kind: Kind) -> Vec<(String, String)> {
    let one = |tag: &str, example: &str| vec![(tag.to_string(), example.to_string())];
    let void = |tag: &str| (tag.to_string(), format!("<{tag} id=\"Name\" />"));
    let mut tags = match kind {
        Kind::Label => vec![
            ("text".into(), "<text id=\"Name\">…</text>".into()),
            ("p".into(), "<p id=\"Name\">Read the <a href=\"rules\">rules</a>.</p>".into()),
            ("span".into(), "<span id=\"Name\">…</span>".into()),
            ("h1".into(), "<h1 id=\"Name\">…</h1> (to <h6>)".into()),
            ("a".into(), "<a id=\"Name\" href=\"shop\">…</a>".into()),
        ],
        Kind::Box => vec![
            ("div".into(), "<div id=\"Name\">…</div>".into()),
            ("nav".into(), "<nav aria-label=\"Site\">…</nav> (section, header, footer, main, aside, article)".into()),
            ("ul".into(), "<ul><li>…</li></ul>".into()),
            ("ol".into(), "<ol start=\"1\"><li>…</li></ol>".into()),
        ],
        Kind::Image => vec![void("img")],
        Kind::Field => vec![
            void("input type=\"text\""),
            (
                "textarea".into(),
                "<textarea id=\"Name\" rows=\"4\" />".into(),
            ),
        ],
        Kind::Slider => vec![void("input type=\"range\"")],
        Kind::Color => vec![(
            "input".into(),
            "<input type=\"color\" id=\"Name\" bind:value={$tint} />".into(),
        )],
        Kind::Radio => vec![(
            "input".into(),
            "<label><input type=\"radio\" value=\"easy\" bind:group={$level} /> Easy</label>".into(),
        )],
        Kind::Toggle => vec![
            void("input type=\"checkbox\""),
            void("input type=\"checkbox\" switch"),
            (
                "label".into(),
                "<label id=\"Name\"><input type=\"checkbox\" /> Caption</label>".into(),
            ),
        ],
        Kind::Divider => vec![void("hr")],
        Kind::Datetime => vec![(
            "input type=\"date\"".into(),
            "<input type=\"date\" id=\"Name\" /> (time, datetime-local, month, week)".into(),
        )],
        Kind::File => vec![void("input type=\"file\"")],
        Kind::Select => vec![(
            "select".into(),
            "<select id=\"Name\" bind:value={$x}><optgroup label=\"G\"><option value=\"a\">A</option></optgroup></select>".into(),
        )],
        Kind::Details => vec![(
            "details".into(),
            "<details id=\"Name\" bind:open={$x}><summary>More</summary>…</details>".into(),
        )],
        Kind::Dialog => vec![(
            "dialog".into(),
            "<dialog id=\"Name\" modal bind:open={$x} aria-label=\"Quit?\">…</dialog>".into(),
        )],
        Kind::Meter => vec![(
            "meter".into(),
            "<meter id=\"Name\" value={$hp} min=\"0\" max=\"100\" low=\"25\" high=\"75\" optimum=\"90\" />".into(),
        )],
        Kind::Icon => Vec::new(),
        Kind::Viewport => one(
            "viewport",
            "<viewport id=\"Preview\" src={$target} camera-orbit=\"30deg 75deg 3m\" bind:clientWidth={$w} onpointermove={orbit} />",
        ),
        Kind::Joystick => one(
            "joystick",
            "<joystick id=\"Stick\" action=\"move\" data-dead-zone=\"0.2\" customize />",
        ),
        Kind::Anchor => one(
            "anchor",
            "<anchor entity={u.entity} offset=\"0 2.1 0\" occlusion=\"fade\" declutter=\"stack\" clamp=\"edge\">…</anchor>",
        ),
        Kind::Dpad => one("dpad", "<dpad id=\"Pad\" action=\"move\" />"),
        Kind::Ground => one(
            "ground",
            "<ground entity={u.entity} size=\"3 3\"><svg viewBox=\"0 0 40 40\">…</svg></ground>",
        ),
        Kind::Svg => one(
            "svg",
            "<svg id=\"Name\" viewBox=\"0 0 40 40\">…</svg>",
        ),
        Kind::SvgText => one(
            "text",
            "<svg viewBox=\"0 0 40 40\"><text x=\"20\" y=\"24\" text-anchor=\"middle\">…</text></svg>",
        ),
        Kind::SvgGroup => one("g", "<g transform=\"rotate(-90 20 20)\" stroke=\"var(--accent)\">…</g>"),
        Kind::SvgUse => one("use", "<use href=\"#Coin\" x={i * 12} y=\"0\" width=\"10\" height=\"10\" />"),
        Kind::SvgForeign => one(
            "foreignObject",
            "<foreignObject x=\"10\" y=\"10\" width=\"80\" height=\"30\"><button onclick={buy}>Buy</button></foreignObject>",
        ),
        Kind::SvgShape => vec![
            ("path".into(), "<path d={$line} fill=\"none\" stroke=\"currentColor\" />".into()),
            ("circle".into(), "<circle cx=\"20\" cy=\"20\" r=\"18\" pathLength=\"1\" stroke-dasharray=\"1 1\" stroke-dashoffset={1 - $ready} />".into()),
            ("rect".into(), "<rect x={b.x} y={40 - b.h} width=\"8\" height={b.h} rx=\"2\" />".into()),
            ("polygon".into(), "<polygon points=\"0,0 10,0 5,8\" /> (ellipse, line, polyline)".into()),
        ],
        Kind::SvgDef => vec![(
            "linearGradient".into(),
            "<linearGradient id=\"Up\" x2=\"0\" y2=\"1\"><stop offset=\"0\" stop-color=\"#fff\" />…</linearGradient> (radialGradient, defs)".into(),
        ), (
            "symbol".into(),
            "<symbol id=\"Coin\" viewBox=\"0 0 24 24\"><circle cx=\"12\" cy=\"12\" r=\"10\" /></symbol> (clipPath, mask, pattern, title)".into(),
        )],
        // Written by its own name.
        Kind::Column
        | Kind::Row
        | Kind::Stack
        | Kind::Grid
        | Kind::Panel
        | Kind::Scroll
        | Kind::List
        | Kind::Cells
        | Kind::Button
        | Kind::Kbd
        | Kind::Art
        | Kind::Progress
        | Kind::Video
        | Kind::Audio
        | Kind::Tabs
        | Kind::Badge
        | Kind::Spacer => vec![(kind.to_string(), format!("<{kind} id=\"Name\">…</{kind}>"))],
    };
    // The `og-` tags of its kind, after HTML's.
    tags.extend(
        ELEMENT_TAGS
            .iter()
            .filter(|(_, k)| *k == kind)
            .map(|(tag, _)| {
                if kind == Kind::Label || kind.container() {
                    (tag.to_string(), format!("<{tag} id=\"Name\">…</{tag}>"))
                } else {
                    void(tag)
                }
            }),
    );
    tags
}
