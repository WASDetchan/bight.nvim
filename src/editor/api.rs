use bight::table::{CellPos, CellRange};
use mlua::{IntoLua, Lua, UserData};
use nvim_oxi as nvim;
use nvim_oxi::lua::Pushable;

use crate::editor::Editor;
use crate::util;

impl UserData for Editor {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("buffer", |_, this| Ok(this.state().buffer.handle()));
    }
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("render", |_, this, ()| {
            this.render();
            Ok(())
        });
        methods.add_method("get_value", |_, this, pos: CellPos| {
            this.state().table.evaluate();
            Ok(this.get_value(pos))
        });
        methods.add_method("get_source", |_, this, pos: CellPos| {
            Ok(this.get_source(pos))
        });
        methods.add_method("set_source", |_, this, (src, pos): (String, CellPos)| {
            this.set_source(pos, src);
            this.render();
            Ok(())
        });
        methods.add_method("get_value_range_as_csv", |_, this, range: CellRange| {
            Ok(this.get_value_range_as_csv(range))
        });
        methods.add_method("set_visual_start", |_, this, pos: CellPos| {
            this.set_visual_start(pos);
            Ok(())
        });
        // IntoLua for CellPos is not yet implemented
        // methods.add_method(
        //     "get_visual_start",
        //     |_, this, ()| Ok(this.get_visual_start()),
        // );
    }
}

impl Pushable for Editor {
    unsafe fn push(
        self,
        lstate: *mut nvim_oxi::lua::ffi::State,
    ) -> Result<std::ffi::c_int, nvim::lua::Error> {
        let state = lstate as *mut mlua::ffi::lua_State;
        let lua = unsafe { Lua::get_or_init_from_ptr(state) };

        lua.exec_raw_lua(|lua| unsafe { self.push_into_stack(lua) })
            .map_err(util::pop_error::<Editor>)?;

        Ok(1)
    }
}
