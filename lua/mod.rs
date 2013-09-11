extern mod extra;
use std::libc::{c_int, c_double};
use std::str::raw;
use std::ptr;
use std::vec::OwnedVector;
use std::c_str::ToCStr;
use std::hashmap::HashMap;
mod luac;

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
	pub fn index_type(&self, index: int) -> LuaType {
		unsafe {
			let t = luac::lua_type(self.state, index as c_int);
			return match t {
				luac::LUA_TNONE => TNone,
				luac::LUA_TNIL => Nil,
				luac::LUA_TBOOLEAN => Boolean,
				luac::LUA_TLIGHTUSERDATA => LightUserData,
				luac::LUA_TNUMBER => Number,
				luac::LUA_TSTRING => String,
				luac::LUA_TTABLE => Table,
				luac::LUA_TFUNCTION => Function,
				luac::LUA_TUSERDATA => UserData,
				luac::LUA_TTHREAD => Thread,
				i => TUnknown(i as int),
			}
		}
	}

	pub fn index_str(&self, index: int) -> ~str {
		match self.index_type(index) {
			TNone         => ~"none: none",
			Nil           => ~"nil: nil",
			Boolean       => fmt!("bool: %?", self.to_bool(index)),
			LightUserData => ~"light user data",
			Number        => fmt!("number: %f", self.to_float(index)),
			String        => fmt!("string: %s", self.to_str(index)),
			Table         => ~"table: <unimplemented>",
			Function      => ~"function: <unimplemented>",
			UserData      => ~"userdata",
			Thread        => ~"thread",
			TUnknown(i)   => fmt!("unknown: %d", i)
		}
	}

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
	pub fn get_top(&self) -> int {
		unsafe {
			luac::lua_gettop(self.state) as int
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
	pub fn insert(&self, index: int) {
		unsafe {
			luac::lua_insert(self.state, index as c_int);
		}
	}

	pub fn new_table(&self) {
		self.create_table(0, 0);
	}

	#[fixed_stack_segment]
	pub fn create_table(&self, narr: int, nrec: int) {
		unsafe {
			luac::lua_createtable(self.state, narr as c_int, nrec as c_int);
		}
	}

	#[fixed_stack_segment]
	pub fn set_table(&self, index: int) {
		unsafe {
			luac::lua_settable(self.state, index as c_int);
		}
	}

	#[fixed_stack_segment]
	pub fn raw_set(&self, index: int) {
		unsafe {
			luac::lua_rawset(self.state, index as c_int);
		}
	}

	#[fixed_stack_segment]
	pub fn raw_set_i(&self, index: int, n: int) {
		unsafe {
			luac::lua_rawseti(self.state, index as c_int, n as c_int);
		}
	}
	
	#[fixed_stack_segment]
	pub fn next(&self, index: int) -> bool {
		unsafe {
			luac::lua_next(self.state, index as c_int) != 0
		}
	}

	pub fn pop(&self, n: int) {
		return self.set_top( -(n) - 1 );
	}

	#[fixed_stack_segment]
	pub fn set_top(&self, index: int) {
		unsafe {
			return luac::lua_settop(self.state, index as c_int);
		}
	}

	#[fixed_stack_segment]
	pub fn to_bool(&self, index: int) -> bool {
		unsafe {
			luac::lua_toboolean(self.state, index as c_int) != 0
		}
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
			luac::lua_tonumber(self.state, index as c_int) as float
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
	pub fn push_bool(&self, b: bool) {
		unsafe {
			luac::lua_pushinteger(self.state, match b {true => 1, false => 0} as c_int);
		}
	}

	#[fixed_stack_segment]
	pub fn push_int(&self, integer: int) {
		unsafe {
			luac::lua_pushinteger(self.state, integer as c_int);
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
			s.with_c_str( |cs| luac::lua_pushstring(self.state, cs));
		}
	}

	#[fixed_stack_segment]
	pub fn push_nil(&self) {
		unsafe {
			luac::lua_pushnil(self.state);
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

pub enum LuaType {
	TNone,
	Nil,
	Boolean,
	LightUserData,
	Number,
	String,
	Table,
	Function,
	UserData,
	Thread,
	TUnknown(int)
}

impl ToStr for LuaType {
	fn to_str(&self) -> ~str {
		match *self {
			TNone           => ~"none",
			Nil             => ~"nil",
			Boolean         => ~"boolean",
			LightUserData   => ~"light user data",
			Number          => ~"number",
			String          => ~"string",
			Table           => ~"table",
			Function        => ~"function",
			UserData        => ~"user data",
			Thread          => ~"thread",
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

pub trait LuaPush {
	fn lua_push(&self, lua: &Lua);
}

pub trait LuaTo {
	fn lua_to(lua: &Lua, index: int) -> Self;
}

impl LuaPush for float {
	fn lua_push(&self, lua: &Lua) {
		lua.push_float(*self);
	}
}

impl LuaTo for float {
	fn lua_to(lua: &Lua, index: int) -> float {
		return lua.to_float(index);
	}
}

impl LuaPush for int {
	fn lua_push(&self, lua: &Lua) {
		lua.push_int(*self);
	}
}

impl LuaTo for int {
	fn lua_to(lua: &Lua, index: int) -> int {
		return lua.to_int(index);
	}
}

impl LuaPush for ~str {
	fn lua_push(&self, lua: &Lua) {
		lua.push_str(*self);
	}
}

impl<'self> LuaPush for &'self str {
	fn lua_push(&self, lua: &Lua) {
		lua.push_str(*self);
	}
}

impl LuaTo for ~str {
	fn lua_to(lua: &Lua, index: int) -> ~str {
		lua.to_str(index)
	}
}

impl<T: LuaPush> LuaPush for ~[T] {
	fn lua_push(&self, lua: &Lua) {
		lua.new_table();

		let mut i: int = 1;
		for v in self.iter() {
			v.lua_push(lua);
			lua.raw_set_i(-2, i);
			i += 1;
		}
	}
}

impl<T: LuaTo> LuaTo for ~[T] {
	fn lua_to(lua: &Lua, index: int) -> ~[T] {
		let mut vect = ~[];

		lua.push_nil();
		while lua.next(index - 1) {
			vect.push( LuaTo::lua_to(lua, index) );
			lua.pop(1);
		}

		return vect;
	}
}

impl<'self, K: LuaPush + Hash + Eq, V: LuaPush> LuaPush for &'self HashMap<K, V> {
	fn lua_push(&self, lua: &Lua) {
		lua.new_table();

		for kv in self.iter() {
			match kv {
				(k, v) => {
					k.lua_push(lua);
					v.lua_push(lua);
					lua.raw_set(-3);
				}
			};
		}
	}
}

impl<K: LuaPush + Hash + Eq, V: LuaPush> LuaPush for HashMap<K, V> {
	fn lua_push(&self, lua: &Lua) {
		lua.new_table();

		for kv in self.iter() {
			match kv {
				(k, v) => {
					k.lua_push(lua);
					v.lua_push(lua);

					lua.raw_set(-3);
				}
			};
		}
	}
}

impl<K: LuaTo + Hash + Eq, V: LuaTo> LuaTo for HashMap<K, V> {
	fn lua_to(lua: &Lua, index: int) -> HashMap<K, V> {
		let mut m: HashMap<K, V> = HashMap::new();

		lua.push_nil();
		if index != -1 { lua.insert(index); }
		while lua.next(index - 1) {
			let v: V = LuaTo::lua_to(lua, index);
			lua.pop(1);

			let k: K = LuaTo::lua_to(lua, index);
			m.swap(k, v);
		}
		return m;
	}
}

pub fn print_stack(lua: &Lua) {
	let top = lua.get_top();
	if top == 0 { println("stack is empty"); return; }

	println("");
	for i in range(1, top + 1) {
		printf!("%d - %s\n", i, lua.index_str(i))
	}
}
