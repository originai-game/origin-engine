//! The document `ui_debug` inspects, shared with the frame test
//! (og-tests-ui includes this file from its `ui_debug`).

pub(crate) const DOC: &str = r#"<meta name="component" content="dbg">
<column id="Root">
  <column id="Card" class="card">
    <text id="Title">{$title}</text>
  </column>
  <scroll id="List" height="80"><text id="Row">Row</text></scroll>
</column>
<style>
.card { width: 220px; height: 90px; padding: 8px 12px; margin: 10px 4px; border: 2px solid #ffffff; }
.card { padding: 8px 12px 8px 16px; }
</style>"#;
