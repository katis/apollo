pub use self::state::*;
use std::hashmap::HashMap;
mod state;

struct Lua {
	priv state: ~state::State
}

pub fn New() -> ~Lua {
	~Lua {
		state: state::NewState()
	}
}

impl Lua {
	pub fn push<T: LuaPush>(&self, p: T) {
		p.lua_push(self);
	}

	pub fn i_to<T: LuaTo>(&self, index: int) -> T {
		LuaTo::lua_to(self, index)
	}

	pub fn pop<T: LuaTo>(&self) -> T {
		let v: T = self.i_to(-1);
		self.state.pop(1);
		return v;
	}
	
	pub fn table_iter<'a, K: LuaTo, V: LuaTo>(&'a self, index: int) -> LuaTableIterator<'a, K, V> {
		match self.state.index_type(index) {
			state::TTable => {},
			_ => { fail!(fmt!("Lua.table_iter() failed, value at index %d is not a table", index)) }
		};
		LuaTableIterator{ lua: self, index: index, started: false, closed: false }
	}

	pub fn arr_iter<'a, T: LuaTo>(&'a self, index: int) -> LuaArrayIterator<'a, T> {
		match self.state.index_type(index) {
			state::TTable => {},
			_ => { fail!(fmt!("Lua.arr_iter() failed, value at index %d is not a table", index)) }
		};
		LuaArrayIterator{ lua: self, index: index, started: false, closed: false }
	}

	pub fn state<'a>(&'a self) -> &'a ~state::State {
		&self.state
	}
}

pub struct LuaArrayIterator<'self, V> {
	priv lua: &'self Lua,
	priv index: int,
	priv started: bool,
	priv closed: bool
}

impl<'self, T: LuaTo> Iterator<T> for LuaArrayIterator<'self, T> {
	fn next(&mut self) -> Option<T> {
		if self.closed {
			return None;
		}
		if !self.started {
			self.lua.state().push_nil();
			self.started = true;
		}
		if !self.lua.state().next(self.index - 1) {
			self.closed = true;
			return None;
		}
		let ret: T = self.lua.i_to(-1);
		self.lua.state().pop(1);
		return Some(ret);
	}
}

pub struct LuaTableIterator<'self, K, V> {
	priv lua: &'self Lua,
	priv index: int,
	priv started: bool,
	priv closed: bool
}

impl<'self, K: LuaTo, V: LuaTo> Iterator<(K, V)> for LuaTableIterator<'self, K, V> {
	fn next(&mut self) -> Option<(K, V)> {
		if self.closed {
			return None;
		}
		if !self.started {
			self.lua.state().push_nil();
			self.started = true;
		}
		if !self.lua.state().next(self.index - 1) {
			self.closed = true;
			return None;
		}
		let ret: (K, V) = (self.lua.i_to(-2), self.lua.i_to(-1));
		self.lua.state().pop(1);
		return Some(ret);
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
		lua.state().push_float(*self);
	}
}

impl<'self> LuaPush for &'self float {
	fn lua_push(&self, lua: &Lua) {
		lua.state().push_float(**self);
	}
}

impl LuaTo for float {
	fn lua_to(lua: &Lua, index: int) -> float {
		return lua.state().to_float(index);
	}
}

impl LuaPush for int {
	fn lua_push(&self, lua: &Lua) {
		lua.state().push_int(*self);
	}
}

impl<'self> LuaPush for &'self int {
	fn lua_push(&self, lua: &Lua) {
		lua.state().push_int(**self);
	}
}

impl LuaTo for int {
	fn lua_to(lua: &Lua, index: int) -> int {
		return lua.state().to_int(index);
	}
}

impl LuaPush for ~str {
	fn lua_push(&self, lua: &Lua) {
		lua.state().push_str(*self);
	}
}

impl<'self> LuaPush for &'self str {
	fn lua_push(&self, lua: &Lua) {
		lua.state().push_str(*self);
	}
}

impl LuaTo for ~str {
	fn lua_to(lua: &Lua, index: int) -> ~str {
		lua.state().to_str(index)
	}
}

impl<T: LuaPush> LuaPush for ~[T] {
	fn lua_push(&self, lua: &Lua) {
		lua.state().new_table();

		let mut i: int = 1;
		for v in self.iter() {
			v.lua_push(lua);
			lua.state().raw_set_i(-2, i);
			i += 1;
		}
	}
}

impl<T: LuaTo> LuaTo for ~[T] {
	fn lua_to(lua: &Lua, index: int) -> ~[T] {
		let mut vect = ~[];

		lua.state().push_nil();
		while lua.state().next(index - 1) {
			vect.push( LuaTo::lua_to(lua, -1) );
			lua.state().pop(1);
		}

		return vect;
	}
}

impl<'self, K: LuaPush + Hash + Eq, V: LuaPush> LuaPush for &'self HashMap<K, V> {
	fn lua_push(&self, lua: &Lua) {
		lua.state().new_table();

		for kv in self.iter() {
			match kv {
				(k, v) => {
					k.lua_push(lua);
					v.lua_push(lua);
					lua.state().raw_set(-3);
				}
			};
		}
	}
}

impl<K: LuaPush + Hash + Eq, V: LuaPush> LuaPush for HashMap<K, V> {
	fn lua_push(&self, lua: &Lua) {
		lua.state().new_table();

		for kv in self.iter() {
			match kv {
				(k, v) => {
					k.lua_push(lua);
					v.lua_push(lua);

					lua.state().raw_set(-3);
				}
			};
		}
	}
}

impl<K: LuaTo + Hash + Eq, V: LuaTo> LuaTo for HashMap<K, V> {
	fn lua_to(lua: &Lua, index: int) -> HashMap<K, V> {
		let mut m: HashMap<K, V> = HashMap::new();

		lua.state().push_nil();
		while lua.state().next(index - 1) {
			let k: K = LuaTo::lua_to(lua, -2);
			let v: V = LuaTo::lua_to(lua, -1);
			lua.state().pop(1);
			m.swap(k, v);
		}
		return m;
	}
}

fn print_stack(state: &state::State) {
	let top = state.get_top();
	if top == 0 { println("stack is empty"); return; }

	printf!("Stack, top: %d\n", top);
	for i in range(1, top + 1) {
		printf!("%d - %s\n", i, state.index_str(i))
	}
}
