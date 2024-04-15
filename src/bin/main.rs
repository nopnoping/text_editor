use std::env;
use text_editor::config::EditorCfg;
use text_editor::editor::Editor;

fn main() {
    let cfg: EditorCfg;
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        cfg = EditorCfg::new(&args[1]);
    } else {
        cfg = EditorCfg::new("");
    }
    let mut editor = Editor::new(cfg);
    editor.run();
}
