use bight::{
    editor::EditorState,
    evaluator::{EvaluatorTable, SourceTable},
};
use nvim_oxi::{
    self as nvim, Dictionary,
    api::{opts::CreateAutocmdOpts, types::AutocmdCallbackArgs},
};

use crate::editor::{EDITORS, add_keymaps, render_buffer};
pub fn attach_editor_autocmd() {
    nvim::api::create_autocmd(
        ["BufReadPost"],
        &CreateAutocmdOpts::builder()
            .patterns(["*.bight"])
            .callback(|args: AutocmdCallbackArgs| {
                let file = args.file;
                let Ok(source) = bight::file::load(&file) else {
                    bight::file::load(&file).unwrap();
                    return false;
                };
                let table = EvaluatorTable::new(source);
                let editor = EditorState {
                    table,
                    ..Default::default()
                };

                let mut buffer = args.buffer;

                add_keymaps(&mut buffer);

                EDITORS.lock().unwrap().insert(buffer.handle(), editor);

                render_buffer(&mut buffer);
                true
            })
            .build(),
    )
    .unwrap();
    nvim::api::create_autocmd(
        ["BufWritePost"],
        &CreateAutocmdOpts::builder()
            .patterns(["*.bight"])
            .callback(|args: AutocmdCallbackArgs| {
                let file = args.file;
                let buffer = args.buffer.handle();
                let editor = EDITORS.lock().unwrap();
                let source = if let Some(state) = editor.get(&buffer) {
                    state.table.source_table().clone()
                } else {
                    SourceTable::new()
                };
                nvim::api::notify(
                    "Saved",
                    nvim_oxi::api::types::LogLevel::Warn,
                    &Dictionary::new(),
                )
                .unwrap();
                bight::file::save(&file, source).is_ok()
            })
            .build(),
    )
    .unwrap();
}
