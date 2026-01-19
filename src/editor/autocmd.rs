use std::{path::Path, sync::Arc};

use bight::{file::BightFile, table::cell::CellPos};
use nvim_oxi::{
    self as nvim,
    api::{
        Buffer,
        opts::{CreateAutocmdOpts, OptionOpts},
        types::AutocmdCallbackArgs,
    },
};

use crate::{
    editor::{Editor, add_keymaps, render_buffer, render_buffer_edit},
    util::{current_cell_pos, get_buffer_as_string, normalize_cursor, notify_err},
};

pub fn init_buffer(mut buffer: Buffer, file: Option<&Path>) {
    let editor = if let Some(file) = file {
        match Editor::with_file_buffer(buffer.clone(), file) {
            Ok(v) => v,

            Err(e) => {
                notify_err(&format!("Failed to load file {:?}: {e}", file));
                Editor::with_new_buffer(buffer.clone())
            }
        }
    } else {
        Editor::with_new_buffer(buffer.clone())
    };

    render_buffer(&editor);

    add_keymaps(&mut buffer, editor.clone());
    attach_buffer_autocmd(buffer, editor);
}

pub fn attach_editor_autocmd() {
    nvim::api::create_autocmd(
        ["BufReadPost"],
        &CreateAutocmdOpts::builder()
            .patterns(["*.bight", "*.csv"])
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
            .patterns(["*.bight", "*.csv"])
            .callback(|args: AutocmdCallbackArgs| {
                init_buffer(args.buffer, None);
                false
            })
            .build(),
    )
    .unwrap();
}

fn attach_buffer_autocmd(buffer: Buffer, editor: Editor) {
    nvim::api::set_option_value(
        "buftype",
        "acwrite",
        &OptionOpts::builder().buf(buffer.clone()).build(),
    )
    .unwrap();
    nvim::api::set_option_value(
        "filetype",
        "bight",
        &OptionOpts::builder().buf(buffer.clone()).build(),
    )
    .unwrap();

    nvim::api::set_option_value(
        "modified",
        false,
        &OptionOpts::builder().buf(buffer.clone()).build(),
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
                    let bfile = BightFile::new(source);
                    if let Err(e) = bight::file::save(&file, &bfile) {
                        notify_err(&format!("Failed to save file {file:?}: {e}"));
                        return false;
                    } else {
                        nvim::api::set_option_value(
                            "modified",
                            false,
                            &OptionOpts::builder().buf(buffer.clone()).build(),
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
                    editor.lock().unwrap().edit = Some(pos);
                    render_buffer_edit(&editor, pos, true);
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

    nvim::api::create_autocmd(
        ["ModeChanged"],
        &CreateAutocmdOpts::builder()
            .buffer(buffer.clone())
            .callback(move |_args: AutocmdCallbackArgs| {
                let mode = nvim::api::get_mode();
                match mode.mode.to_string().as_str() {
                    "n" => normalize_cursor(),
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
                editor.exit_edit();
                render_buffer(&editor);
                false
            })
            .buffer(buffer.clone())
            .build(),
    )
    .unwrap();
}

pub fn attach_cell_edit_autocmd(pos: CellPos, buffer: Buffer, editor: Editor) {
    nvim::api::create_autocmd(
        ["BufWriteCmd"],
        &CreateAutocmdOpts::builder()
            .buffer(buffer.clone())
            .callback(move |_args: AutocmdCallbackArgs| {
                let content = get_buffer_as_string(&buffer);
                nvim::api::set_option_value(
                    "modified",
                    false,
                    &OptionOpts::builder().buf(buffer.clone()).build(),
                )
                .unwrap();
                editor
                    .state()
                    .table
                    .set_source(pos, Some(Arc::from(content)));
                let editor_buf = editor.lock().unwrap().buffer.clone();
                render_buffer(&editor);
                nvim::api::set_option_value(
                    "modified",
                    true,
                    &OptionOpts::builder().buf(editor_buf.clone()).build(),
                )
                .unwrap();
                false
            })
            .build(),
    )
    .unwrap();
}
