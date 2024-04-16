use std::env;
use text_editor::config::EditorCfg;
use text_editor::editor::Editor;

fn main() {
    let cfg: EditorCfg;
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        cfg = EditorCfg::new(args[1].clone());
    } else {
        cfg = EditorCfg::new(String::new());
    }
    let mut editor = Editor::new(cfg);
    editor.run();
}
