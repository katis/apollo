extern mod extra;
use std::libc::{c_int, c_double};
use std::str::raw;
use std::ptr;
use std::c_str::ToCStr;
pub use self::ffi::*;
mod ffi;

struct State {
	priv state: *ffi::lua_State
}

#[fixed_stack_segment] #[inline(never)]
pub fn NewState() -> State {
	unsafe {
		let state = ffi::luaL_newstate();
		return State { state: state };
	}
}

pub fn with_state<'r>(raw: *ffi::lua_State, f: &'r fn(&State)) {
	f(&State{ state: raw });
}

impl State {
	#[fixed_stack_segment] #[inline(never)]
	pub fn close(&self) {
		unsafe {
			ffi::lua_close(self.state);
		}
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn open_libs(&self) {
		unsafe {
			ffi::luaL_openlibs(self.state);
		}
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn index_type(&self, index: int) -> LuaType {
		unsafe {
			let t = ffi::lua_type(self.state, index as c_int);
			return match t {
				ffi::LUA_TNONE          => TNone,
				ffi::LUA_TNIL           => TNil,
				ffi::LUA_TBOOLEAN       => TBoolean,
				ffi::LUA_TLIGHTUSERDATA => TLightUserData,
				ffi::LUA_TNUMBER        => TNumber,
				ffi::LUA_TSTRING        => TString,
				ffi::LUA_TTABLE         => TTable,
				ffi::LUA_TFUNCTION      => TFunction,
				ffi::LUA_TUSERDATA      => TUserData,
				ffi::LUA_TTHREAD        => TThread,
				i                        => TUnknown(i as int),
			}
		}
	}

	pub fn index_str(&self, index: int) -> ~str {
		match self.index_type(index) {
			TNone         => ~"none: none",
			TNil           => ~"nil: nil",
			TBoolean       => fmt!("bool: %?", self.to_bool(index)),
			TLightUserData => ~"light user data",
			TNumber        => fmt!("number: %f", self.to_float(index)),
			TString        => fmt!("string: %s", self.to_str(index)),
			TTable         => ~"table",
			TFunction      => ~"function",
			TUserData      => ~"userdata",
			TThread        => ~"thread",
			TUnknown(i)   => fmt!("unknown: %d", i)
		}
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn pcall(&self, nargs: int, nresults: int, errfunci: int) {
		unsafe {
			let err = self.maybe_err(ffi::lua_pcall(self.state,
				nargs as c_int, nresults as c_int, errfunci as c_int));
			match err {
				Some(msg) => {
					fail!(fmt!("pcall failed: %s", msg.to_str()));
				},
				_ => {}
			};
		}
	}

	pub fn get_global(&self, name: &str) {
		self.get_field(ffi::LUA_GLOBALSINDEX as int, name);
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn get_field(&self, index: int, name: &str) {
		unsafe {
			let c_name = name.to_c_str();
			ffi::lua_getfield(self.state, index as c_int, c_name.unwrap());
		}
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn get_top(&self) -> int {
		unsafe {
			ffi::lua_gettop(self.state) as int
		}
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn load_file(&self, filename: &str) {
		unsafe {
			let cfname = filename.to_c_str();
			let err = self.maybe_err(ffi::luaL_loadfile(self.state, cfname.unwrap()));
			match err {
				Some(msg) => { fail!(fmt!("load_file failed: %s", msg.to_str())); },
				_ => {}
			}
		}
	}

	pub fn do_file(&self, filename: &str) {
		self.load_file(filename);
		self.pcall(0, ffi::LUA_MULTRET as int, 0);
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn do_str(&self, s: &str) {
		unsafe {
			s.with_c_str( |cs| ffi::luaL_loadstring(self.state, cs) );
		}

		self.pcall(0, ffi::LUA_MULTRET as int, 0);
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn insert(&self, index: int) {
		unsafe {
			ffi::lua_insert(self.state, index as c_int);
		}
	}

	pub fn new_table(&self) {
		self.create_table(0, 0);
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn create_table(&self, narr: int, nrec: int) {
		unsafe {
			ffi::lua_createtable(self.state, narr as c_int, nrec as c_int);
		}
	}

	pub fn set_global(&self, name: &str) {
		self.set_field(ffi::LUA_GLOBALSINDEX as int, name);
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn set_field(&self, index: int, name: &str) {
		unsafe {
			name.with_c_str( |n| ffi::lua_setfield(self.state, index as c_int, n) );
		}
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn set_table(&self, index: int) {
		unsafe {
			ffi::lua_settable(self.state, index as c_int);
		}
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn raw_set(&self, index: int) {
		unsafe {
			ffi::lua_rawset(self.state, index as c_int);
		}
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn raw_set_i(&self, index: int, n: int) {
		unsafe {
			ffi::lua_rawseti(self.state, index as c_int, n as c_int);
		}
	}
	
	#[fixed_stack_segment] #[inline(never)]
	pub fn next(&self, index: int) -> bool {
		unsafe {
			ffi::lua_next(self.state, index as c_int) != 0
		}
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn remove(&self, index: int) {
		unsafe {
			ffi::lua_remove(self.state, index as c_int);
		}
	}

	pub fn pop(&self, n: int) {
		return self.set_top( -(n) - 1 );
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn set_top(&self, index: int) {
		unsafe {
			return ffi::lua_settop(self.state, index as c_int);
		}
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn to_bool(&self, index: int) -> bool {
		unsafe {
			match self.index_type(index) {
				TBoolean => {
					return ffi::lua_toboolean(self.state, index as c_int) != 0
				},
				t => {
					return fail!(fmt!("to_bool failed because stack has %s", t.to_str()));
				}
			};
		}
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn to_int(&self, index: int) -> int {
		unsafe {
			match self.index_type(index) {
				TNumber => {
					return ffi::lua_tointeger(self.state, index as c_int) as int;
				},
				t => {
					return fail!(fmt!("to_int failed because stack has %s", t.to_str()));
				}
			};
		}
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn to_str(&self, index: int) -> ~str{
		unsafe {
			match self.index_type(index) {
				TString => {
					let strPtr = ffi::lua_tolstring(self.state, index as c_int, ptr::null());
					return raw::from_c_str(strPtr);
				},
				t => {
					return fail!(fmt!("to_str failed because stack has %s", t.to_str()));
				}
			}
		}
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn to_float(&self, index: int) -> float {
		unsafe {
			match self.index_type(index) {
				TNumber => {
					return ffi::lua_tonumber(self.state, index as c_int) as float;
				},
				t => {
					return fail!(fmt!("to_float failed because stack has %s", t.to_str()));
				}
			}
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
			ffi::LUA_YIELD     => Yield(msg),
			ffi::LUA_ERRRUN    => Runtime(msg),
			ffi::LUA_ERRSYNTAX => Syntax(msg),
			ffi::LUA_ERRMEM    => MemAlloc(msg),
			ffi::LUA_ERRERR    => ErrFunc(msg),
			_ => Unknown(msg)
		};
		return err;
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn push_bool(&self, b: bool) {
		unsafe {
			ffi::lua_pushinteger(self.state, match b {true => 1, false => 0} as c_int);
		}
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn push_int(&self, integer: int) {
		unsafe {
			ffi::lua_pushinteger(self.state, integer as c_int);
		}
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn push_float(&self, a: float) {
		unsafe {
			ffi::lua_pushnumber(self.state, a as c_double);
		}
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn push_str(&self, s: &str) {
		unsafe {
			s.with_c_str( |cs| ffi::lua_pushstring(self.state, cs));
		}
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn push_function(&self, f: ffi::LuaCallback) {
		unsafe {
			ffi::lua_pushcclosure(self.state, f, 0);
		}
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn push_closure(&self, cb: ffi::LuaCallback, upvals: int) {
		unsafe {
			ffi::lua_pushcclosure(self.state, cb, upvals as c_int);
		}
	}

	#[fixed_stack_segment] #[inline(never)]
	pub fn push_nil(&self) {
		unsafe {
			ffi::lua_pushnil(self.state);
		}
	}
}

pub enum LuaType {
	TNone,
	TNil,
	TBoolean,
	TLightUserData,
	TNumber,
	TString,
	TTable,
	TFunction,
	TUserData,
	TThread,
	TUnknown(int)
}

impl ToStr for LuaType {
	fn to_str(&self) -> ~str {
		match *self {
			TNone           => ~"none",
			TNil             => ~"nil",
			TBoolean         => ~"boolean",
			TLightUserData   => ~"light user data",
			TNumber          => ~"number",
			TString          => ~"string",
			TTable           => ~"table",
			TFunction        => ~"function",
			TUserData        => ~"user data",
			TThread          => ~"thread",
			TUnknown(ref t) => fmt!("unknown: %d", *t)
		}
	}
}

pub enum LuaErr {
	Yield(~str),
	Runtime(~str),
	Syntax(~str),
	MemAlloc(~str),
	ErrFunc(~str),
	Unknown(~str)
}

impl ToStr for LuaErr {
	fn to_str(&self) -> ~str {
		match *self {
			Yield(ref msg)    => fmt!("Lua yield error: %s", *msg),
			Runtime(ref msg)  => fmt!("Lua runtime error: %s", *msg),
			Syntax(ref msg)   => fmt!("Lua syntax error: %s", *msg),
			MemAlloc(ref msg) => fmt!("Lua memory allocation error: %s", *msg),
			ErrFunc(ref msg)  => fmt!("Lua error handler error: %s", *msg),
			Unknown(ref msg)  => fmt!("Lua error: %s", *msg),
		}
	}
}
