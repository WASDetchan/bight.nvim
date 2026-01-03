use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use bight::table::cell::CellPos;
use nvim_oxi::{
    self as nvim,
    api::{
        Buffer,
        opts::{CreateAutocmdOpts, OptionOpts},
        types::AutocmdCallbackArgs,
    },
};

use crate::editor::{
    CELL_UNIT_WIDTH, Editor, EditorState, add_keymaps, exit_insert, render_buffer,
    render_buffer_replace,
    util::{current_cell_pos, notify_err},
};

pub fn init_buffer(mut buffer: Buffer, file: Option<&Path>) {
    let editor = if let Some(file) = file {
        match EditorState::with_file_buffer(buffer.clone(), file) {
            Ok(v) => v,

            Err(e) => {
                notify_err(&format!("Failed to load file {:?}: {e}", file));
                EditorState::with_new_buffer(buffer.clone())
            }
        }
    } else {
        EditorState::with_new_buffer(buffer.clone())
    };

    render_buffer(editor.clone());

    add_keymaps(&mut buffer, editor.clone());
    attach_buffer_autocmd(buffer, editor);
}

pub fn attach_editor_autocmd() {
    nvim::api::create_autocmd(
        ["BufReadPost"],
        &CreateAutocmdOpts::builder()
            .patterns(["*.bight"])
            .callback(|args: AutocmdCallbackArgs| {
                init_buffer(args.buffer, Some(&args.file));
                false
            })
            .build(),
    )
    .unwrap();
    nvim::api::create_autocmd(
        ["BufNewFile"],
        &CreateAutocmdOpts::builder()
            .patterns(["*.bight"])
            .callback(|args: AutocmdCallbackArgs| {
                init_buffer(args.buffer, None);
                false
            })
            .build(),
    )
    .unwrap();
}

fn attach_buffer_autocmd(buffer: Buffer, editor: Arc<Mutex<EditorState>>) {
    nvim::api::set_option_value(
        "buftype",
        "acwrite",
        &OptionOpts::builder().buffer(buffer.clone()).build(),
    )
    .unwrap();
    nvim::api::set_option_value(
        "filetype",
        "bight",
        &OptionOpts::builder().buffer(buffer.clone()).build(),
    )
    .unwrap();

    nvim::api::set_option_value(
        "modified",
        false,
        &OptionOpts::builder().buffer(buffer.clone()).build(),
    )
    .unwrap();
    {
        let editor = editor.clone();
        let buffer = buffer.clone();
        let cbuffer = buffer.clone();
        nvim::api::create_autocmd(
            ["BufWriteCmd"],
            &CreateAutocmdOpts::builder()
                .callback(move |args: AutocmdCallbackArgs| {
                    let file = args.file;
                    let source = editor.lock().unwrap().table.source_table().clone();
                    if let Err(e) = bight::file::save(&file, source) {
                        notify_err(&format!("Failed to save file {file:?}: {e}"));
                        return false;
                    } else {
                        nvim::api::set_option_value(
                            "modified",
                            false,
                            &OptionOpts::builder().buffer(buffer.clone()).build(),
                        )
                        .unwrap();
                    }
                    false
                })
                .buffer(cbuffer)
                .build(),
        )
        .unwrap();
    }

    {
        let editor = editor.clone();
        nvim::api::create_autocmd(
            ["InsertEnter"],
            &CreateAutocmdOpts::builder()
                .callback(move |_args: AutocmdCallbackArgs| {
                    let pos = current_cell_pos();
                    editor.lock().unwrap().replace_mode = Some(pos);
                    render_buffer_replace(editor.clone(), pos, true);
                    false
                })
                .buffer(buffer.clone())
                .build(),
        )
        .unwrap();
    }

    let cb_to_v = {
        let editor = editor.clone();
        move || {
            editor.lock().unwrap().visual_start = current_cell_pos();
        }
    };
    let cb_to_n = {
        move || {
            let cell_pos = current_cell_pos();
            nvim::api::get_current_win()
                .set_cursor(cell_pos.y + 1, cell_pos.x * CELL_UNIT_WIDTH)
                .unwrap();
        }
    };

    nvim::api::create_autocmd(
        ["ModeChanged"],
        &CreateAutocmdOpts::builder()
            .buffer(buffer.clone())
            .callback(move |_args: AutocmdCallbackArgs| {
                let mode = nvim::api::get_mode().unwrap();
                eprintln!("ModeChanged to {:?}", mode.mode);
                match mode.mode.to_string().as_str() {
                    "n" => cb_to_n(),
                    "\u{16}" => cb_to_v(),
                    _ => {}
                };
                false
            })
            .build(),
    )
    .unwrap();

    nvim::api::create_autocmd(
        ["InsertLeave"],
        &CreateAutocmdOpts::builder()
            .callback(move |_args: AutocmdCallbackArgs| {
                exit_insert(&editor);
                render_buffer(editor.clone());
                false
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

    nvim::api::set_option_value(
        "modified",
        false,
        &OptionOpts::builder().buffer(buffer.clone()).build(),
    )
    .unwrap();

    nvim::api::create_autocmd(
        ["BufWriteCmd"],
        &CreateAutocmdOpts::builder()
            .buffer(buffer.clone())
            .callback(move |_args: AutocmdCallbackArgs| {
                let content = buffer
                    .get_lines(.., false)
                    .unwrap()
                    .fold(String::new(), |v, a| format!("{v}{a}\n"));
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
