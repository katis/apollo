use std::libc::{c_void, size_t, c_int, c_char, c_double};

// Option for multiple returns in `lua_pcall' and `lua_call'
pub static LUA_MULTRET: c_int = -1;

// Thread Status / Error values
pub static LUA_YIELD:     c_int = 1;
pub static LUA_ERRRUN:    c_int = 2;
pub static LUA_ERRSYNTAX: c_int = 3;
pub static LUA_ERRMEM:    c_int = 4;
pub static LUA_ERRERR:    c_int = 5;

// Pseudo-indices
pub static LUA_REGISTRYINDEX: c_int = -10000;
pub static LUA_ENVIRONINDEX:  c_int = -10001;
pub static LUA_GLOBALSINDEX:  c_int = -10002;

// Lua Types
pub static LUA_TNONE:          c_int = -1;
pub static LUA_TNIL:           c_int = 0;
pub static LUA_TBOOLEAN:       c_int = 1;
pub static LUA_TLIGHTUSERDATA: c_int = 2;
pub static LUA_TNUMBER:        c_int = 3;
pub static LUA_TSTRING:        c_int = 4;
pub static LUA_TTABLE:         c_int = 5;
pub static LUA_TFUNCTION:      c_int = 6;
pub static LUA_TUSERDATA:      c_int = 7;
pub static LUA_TTHREAD:        c_int = 8;

pub type lua_State = c_void;
pub type lua_Number = c_double;
pub type LuaCallback = extern "C" fn(*lua_State) -> c_int;

#[link_args = "-lluajit-5.1"]
extern {
	fn luaL_newstate() -> *lua_State;
	fn lua_close(L: *lua_State);

	fn luaL_openlibs(L: *lua_State);
	fn luaL_loadfile(L: *lua_State, filename: *c_char) -> c_int;
	fn luaL_loadstring(L: *lua_State, s: *c_char) -> c_int;

	fn lua_getfield(L: *lua_State, index: c_int, name: *c_char);
	fn lua_pcall(L: *lua_State, nargs: c_int, nresults: c_int, errfunc: c_int) -> c_int;
	fn lua_type(L: *lua_State, index: c_int) -> c_int;
	fn lua_gettop(L: *lua_State) -> c_int;

	fn lua_insert(L: *lua_State, index: c_int);

	fn lua_createtable(L: *lua_State, narr: c_int, nrec: c_int);
	fn lua_newtable(L: *lua_State);
	fn lua_settable(L: *lua_State, index: c_int);
	fn lua_next(L: *lua_State, index: c_int) -> c_int;

	fn lua_setfield(L: *lua_State, index: c_int, name: *c_char);
	fn lua_rawset(L: *lua_State, index: c_int);
	fn lua_rawseti(L: *lua_State, index: c_int, n: c_int);

	fn lua_pushboolean(L: *lua_State, boolean: c_int);
	fn lua_pushinteger(L: *lua_State, integer: c_int);
	fn lua_pushnumber(L: *lua_State, number: lua_Number);
	fn lua_pushlstring(L: *lua_State, string: *c_char, len: size_t);
	fn lua_pushstring(L: *lua_State, string: *c_char);
	fn lua_pushcclosure(L: *lua_State, cb: LuaCallback, upvals: c_int);
	fn lua_pushnil(L: *lua_State);

	fn lua_isfunction(L: *lua_State, index: c_int) -> c_int;
	fn lua_isnumber(L: *lua_State, index: c_int) -> c_int;
	fn lua_isstring(L: *lua_State, index: c_int) -> c_int;
	fn lua_isnil(L: *lua_State, index: c_int) -> c_int;

	fn lua_remove(L: *lua_State, index: c_int);
	fn lua_settop(L: *lua_State, index: c_int);

	fn lua_toboolean(L: *lua_State, index: c_int) -> c_int;
	fn lua_tointeger(L: *lua_State, index: c_int) -> c_int;
	fn lua_tonumber(L: *lua_State, index: c_int) -> lua_Number;
	fn lua_tolstring(L: *lua_State, index: c_int, len: *size_t) -> *c_char;
}
