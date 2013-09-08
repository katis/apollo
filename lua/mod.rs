use std::libc::{c_int, c_double};
use std::str::raw;
use std::ptr;
mod luac;

pub trait LuaVal {
	fn push(self, lua: &Lua);
	fn pop(lua: &Lua) -> Self;
}

impl LuaVal for float {
	fn push(self, lua: &Lua) {
		lua.push_float(self);
	}
	fn pop(lua: &Lua) -> float {
		lua.to_float(-1)
	}
}

impl LuaVal for int {
	fn push(self, lua: &Lua) {
		lua.push_int(self);
	}
	fn pop(lua: &Lua) -> int {
		lua.to_int(-1)
	}
}

impl LuaVal for ~str {
	fn push(self, lua: &Lua) {
		lua.push_str(self);
	}
	fn pop(lua: &Lua) -> ~str {
		lua.to_str(-1)
	}
}

struct Lua {
	state: *luac::lua_State
}

#[fixed_stack_segment]
pub fn New() -> ~Lua {
	unsafe {
		let state = luac::luaL_newstate();
		luac::luaL_openlibs(state);
		return ~Lua { state: state };
	}
}

impl Lua {
	#[fixed_stack_segment]
	pub fn pcall(&self, nargs: int, nresults: int, errfunci: int) -> Option<LuaErr> {
		unsafe {
			let err = luac::lua_pcall(self.state,
				nargs as c_int, nresults as c_int, errfunci as c_int);
			return self.maybe_err(err);
		}
	}

	pub fn get_global(&self, name: &str) {
		self.get_field(luac::LUA_GLOBALSINDEX as int, name);
	}

	#[fixed_stack_segment]
	pub fn get_field(&self, index: int, name: &str) {
		unsafe {
			let c_name = name.to_c_str();
			luac::lua_getfield(self.state, index as c_int, c_name.unwrap());
		}
	}

	#[fixed_stack_segment]
	pub fn load_file(&self, filename: &str) -> Option<LuaErr> {
		unsafe {
			let cfname = filename.to_c_str();
			return self.maybe_err(luac::luaL_loadfile(self.state, cfname.unwrap()));
		}
	}

	pub fn do_file(&self, filename: &str) -> Option<LuaErr> {
		let err = self.load_file(filename);
		match err {
			Some(_) => { return err; },
			_ => {}
		}

		return self.pcall(0, luac::LUA_MULTRET as int, 0);
	}

	#[fixed_stack_segment]
	pub fn to_int(&self, index: int) -> int {
		unsafe {
			return luac::lua_tointeger(self.state, index as c_int) as int;
		}
	}

	#[fixed_stack_segment]
	pub fn to_str(&self, index: int) -> ~str{
		unsafe {
			let strPtr = luac::lua_tolstring(self.state, index as c_int, ptr::null());
			return raw::from_c_str(strPtr);
		}
	}

	#[fixed_stack_segment]
	pub fn to_float(&self, index: int) -> float {
		unsafe {
			return luac::lua_tonumber(self.state, index as c_int) as float;
		}
	}

	pub fn maybe_err(&self, errn: c_int) -> Option<LuaErr> {
		if errn == 0 { return None; }
		return Some(self.pop_err(errn));
	}

	pub fn pop_err(&self, errn: c_int) -> LuaErr {
		assert!(errn != 0);

		let msg = self.to_str(-1);
		let err = match errn {
			luac::LUA_YIELD     => Yield(msg),
			luac::LUA_ERRRUN    => Runtime(msg),
			luac::LUA_ERRSYNTAX => Syntax(msg),
			luac::LUA_ERRMEM    => MemAlloc(msg),
			luac::LUA_ERRERR    => ErrFunc(msg),
			_ => Unknown
		};
		return err;
	}

	#[fixed_stack_segment]
	pub fn push_int(&self, index: int) {
		unsafe {
			luac::lua_pushinteger(self.state, index as c_int);
		}
	}

	#[fixed_stack_segment]
	pub fn push_float(&self, a: float) {
		unsafe {
			luac::lua_pushnumber(self.state, a as c_double);
		}
	}

	#[fixed_stack_segment]
	pub fn push_str(&self, s: &str) {
		unsafe {
			let cs = s.to_c_str();
			luac::lua_pushstring(self.state, cs.unwrap());
		}
	}
}

impl Drop for Lua {
	#[fixed_stack_segment]
	fn drop(&self) {
		unsafe {
			luac::lua_close(self.state);
		}
	}
}

pub enum LuaErr {
	Yield(~str),
	Runtime(~str),
	Syntax(~str),
	MemAlloc(~str),
	ErrFunc(~str),
	Unknown
}

impl ToStr for LuaErr {
	fn to_str(&self) -> ~str {
		match *self {
			Yield(ref msg)    => fmt!("Lua yield error: %s", *msg),
			Runtime(ref msg)  => fmt!("Lua runtime error: %s", *msg),
			Syntax(ref msg)   => fmt!("Lua syntax error: %s", *msg),
			MemAlloc(ref msg) => fmt!("Lua memory allocation error: %s", *msg),
			ErrFunc(ref msg)  => fmt!("Lua error handler error: %s", *msg),
			Unknown           => fmt!("Lua unknown error"),
		}
	}
}
