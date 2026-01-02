use std::sync::{Arc, Mutex};

use bight::{clipboard::Clipboard, evaluator::EvaluatorTable, table::cell::CellPos};
use nvim_oxi::{
    self as nvim, Dictionary,
    api::{
        Buffer,
        opts::{CreateAutocmdOpts, OptionOpts},
        types::{AutocmdCallbackArgs, LogLevel},
    },
};

use crate::editor::{Editor, EditorState, add_keymaps, render_buffer};

pub fn buf_create(args: AutocmdCallbackArgs) -> Option<Arc<Mutex<EditorState>>> {
    let file = args.file;
    let res = bight::file::load(&file);
    let Ok(source) = res else {
        nvim::api::notify(
            &format!("Faile to load file {file:?}: {}", res.unwrap_err()),
            LogLevel::Error,
            &Dictionary::new(),
        )
        .unwrap();
        return None;
    };
    let table = EvaluatorTable::new(source);
    let editor = Arc::new(Mutex::new(EditorState {
        table,
        buffer: args.buffer,
        clipboard: Clipboard::default(),
    }));

    render_buffer(editor.clone());

    Some(editor)
}

pub fn attach_editor_autocmd() {
    nvim::api::create_autocmd(
        ["BufReadPost"],
        &CreateAutocmdOpts::builder()
            .patterns(["*.bight"])
            .callback(|mut args: AutocmdCallbackArgs| {
                let buffer = args.buffer.clone();
                eprintln!("attach_editor_autocmd");
                let Some(editor) = buf_create(args.clone()) else {
                    eprintln!("ohno");
                    return false;
                };
                attach_buffer_autocmd(buffer, editor.clone());
                add_keymaps(&mut args.buffer, editor);
                true
            })
            .build(),
    )
    .unwrap();
}

fn attach_buffer_autocmd(buffer: Buffer, editor: Arc<Mutex<EditorState>>) {
    nvim::api::create_autocmd(
        ["BufWritePost"],
        &CreateAutocmdOpts::builder()
            .callback(move |args: AutocmdCallbackArgs| {
                let file = args.file;
                let source = editor.lock().unwrap().table.source_table().clone();
                bight::file::save(&file, source).is_ok()
            })
            .buffer(buffer.clone())
            .build(),
    )
    .unwrap();
}

pub fn start_editing_cell(pos: CellPos, editor: Editor) {
    let source = editor
        .lock()
        .unwrap()
        .table
        .get_source(pos)
        .map(|s| s.lines().map(String::from).collect::<Vec<_>>())
        .unwrap_or_default();
    let mut buffer = nvim::api::create_buf(true, false).unwrap();
    buffer.set_lines(.., false, source).unwrap();
    buffer
        .set_name(format!(
            "{pos}-{}",
            std::time::SystemTime::UNIX_EPOCH
                .elapsed()
                .unwrap()
                .as_millis()
        ))
        .unwrap();
    nvim::api::set_option_value(
        "buftype",
        "acwrite",
        &OptionOpts::builder().buffer(buffer.clone()).build(),
    )
    .unwrap();
    nvim::api::set_option_value(
        "filetype",
        "bcell",
        &OptionOpts::builder().buffer(buffer.clone()).build(),
    )
    .unwrap();

    nvim::api::create_autocmd(
        ["BufWriteCmd"],
        &CreateAutocmdOpts::builder()
            .buffer(buffer.clone())
            .callback(move |_args: AutocmdCallbackArgs| {
                eprintln!("BufWriteCmd");
                let content = buffer
                    .get_lines(.., false)
                    .unwrap()
                    .fold(String::new(), |v, a| format!("{v}{a}\n"));
                eprintln!("saved {pos}, content: {content}");
                nvim::api::set_option_value(
                    "modified",
                    false,
                    &OptionOpts::builder().buffer(buffer.clone()).build(),
                )
                .unwrap();
                editor
                    .lock()
                    .unwrap()
                    .table
                    .set_source(pos, Some(Arc::from(content)));
                let editor_buf = editor.lock().unwrap().buffer.clone();
                render_buffer(editor.clone());
                nvim::api::set_option_value(
                    "modified",
                    true,
                    &OptionOpts::builder().buffer(editor_buf.clone()).build(),
                )
                .unwrap();
                false
            })
            .build(),
    )
    .unwrap();
}
