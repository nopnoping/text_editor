use text_editor::config::EditorCfg;
use text_editor::editor::Editor;

fn main() {
    let mut editor = Editor::new(EditorCfg::new());
    editor.run();
}
