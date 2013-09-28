pub use self::state::*;
pub use ffi::LuaCallback;
use std::hashmap::HashMap;
mod state;

struct Lua {
	priv state: state::State
}

pub fn New() -> Lua {
	Lua {
		state: state::NewState()
	}
}

impl Lua {
	///Push a value to the Lua stack.
	pub fn push<T: LuaPush>(&self, p: T) {
		p.lua_push(&self.state);
	}

	/**
	 * Get a value from a Lua stack index.
	 *
	 * Fails if the value in index is the wrong type.
	 */
	pub fn i_to<T: LuaTo>(&self, index: int) -> T {
		LuaTo::lua_to(&self.state, index)
	}

	/**
	 * Pop a top value from the stack and returns it.
	 *
	 * Fails if the value in the top index is the wrong type.
	 */
	pub fn pop<T: LuaTo>(&self) -> T {
		let v: T = self.i_to(-1);
		self.state.pop(1);
		return v;
	}
	
	/**
	 * Get a (Key, Value) iterator for the table at stack index.
	 *
	 * Fails if the value in index is not a table.
	 */
	pub fn table_iter<'a, K: LuaTo, V: LuaTo>(&'a self, index: int) -> LuaTableIterator<'a, K, V> {
		match self.state.index_type(index) {
			state::TTable => {},
			_ => { fail!(fmt!("Lua.table_iter() failed, value at index %d is not a table", index)) }
		};
		LuaTableIterator{ lua: self, index: index, started: false, closed: false }
	}

	/**
	 * Get a value iterator for the table at stack index.
	 *
	 * Fails if the value in index is not a table.
	 */
	pub fn arr_iter<'a, T: LuaTo>(&'a self, index: int) -> LuaArrayIterator<'a, T> {
		match self.state.index_type(index) {
			state::TTable => {},
			_ => { fail!(fmt!("Lua.arr_iter() failed, value at index %d is not a table", index)) }
		};
		LuaArrayIterator{ lua: self, index: index, started: false, closed: false }
	}

	pub fn module<'l>(&'l self, mod_name: &str, def_fn: &fn(&LuaModule<'l>)) {
		self.state().new_table();
		let m = LuaModule{lua: self, table_i: self.state().get_top()};
		def_fn(&m);
		self.state().set_global(mod_name);
	}

	/// Get a borrowed reference to the Lua state.
	pub fn state<'a>(&'a self) -> &'a state::State {
		&self.state
	}
}

#[unsafe_destructor]
impl Drop for Lua {
	#[fixed_stack_segment] #[inline(never)]
	fn drop(&mut self) {
		self.state.close();
	}
}

pub struct LuaModule<'self> {
	priv lua: &'self Lua,
	priv table_i: int
}

impl<'self> LuaModule<'self> {
	pub fn namespace(&self, name: &str, def_fn: &fn(&LuaModule<'self>)) {
		self.lua.state().new_table();
		let ns = LuaModule{ lua: self.lua, table_i: self.lua.state().get_top() };
		def_fn(&ns);
		self.lua.state().set_field(self.table_i, name);
	}

	pub fn def<T: LuaPush>(&self, name: &str, val: T) {
		self.lua.push(name);
		self.lua.push(val);
		self.lua.state().raw_set(-3);
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
	fn lua_push(&self, state: &state::State);
}

pub trait LuaTo {
	fn lua_to(state: &state::State, index: int) -> Self;
}

impl LuaPush for float {
	fn lua_push(&self, state: &state::State) {
		state.push_float(*self);
	}
}

impl<'self> LuaPush for &'self float {
	fn lua_push(&self, state: &state::State) {
		state.push_float(**self);
	}
}

impl LuaTo for float {
	fn lua_to(state: &state::State, index: int) -> float {
		return state.to_float(index);
	}
}

impl LuaPush for int {
	fn lua_push(&self, state: &state::State) {
		state.push_int(*self);
	}
}

impl<'self> LuaPush for &'self int {
	fn lua_push(&self, state: &state::State) {
		state.push_int(**self);
	}
}

impl LuaTo for int {
	fn lua_to(state: &state::State, index: int) -> int {
		return state.to_int(index);
	}
}

impl LuaPush for ~str {
	fn lua_push(&self, state: &state::State) {
		state.push_str(*self);
	}
}

impl<'self> LuaPush for &'self str {
	fn lua_push(&self, state: &state::State) {
		state.push_str(*self);
	}
}

impl LuaTo for ~str {
	fn lua_to(state: &state::State, index: int) -> ~str {
		state.to_str(index)
	}
}

impl<T: LuaPush> LuaPush for ~[T] {
	fn lua_push(&self, state: &state::State) {
		state.new_table();

		let mut i: int = 1;
		for v in self.iter() {
			v.lua_push(state);
			state.raw_set_i(-2, i);
			i += 1;
		}
	}
}

impl<T: LuaTo> LuaTo for ~[T] {
	fn lua_to(state: &state::State, index: int) -> ~[T] {
		let mut vect = ~[];

		state.push_nil();
		while state.next(index - 1) {
			vect.push( LuaTo::lua_to(state, -1) );
			state.pop(1);
		}

		return vect;
	}
}

impl<'self, K: LuaPush + Hash + Eq, V: LuaPush> LuaPush for &'self HashMap<K, V> {
	fn lua_push(&self, state: &state::State) {
		state.new_table();

		for kv in self.iter() {
			match kv {
				(k, v) => {
					k.lua_push(state);
					v.lua_push(state);
					state.raw_set(-3);
				}
			};
		}
	}
}

impl<K: LuaPush + Hash + Eq, V: LuaPush> LuaPush for HashMap<K, V> {
	fn lua_push(&self, state: &state::State) {
		state.new_table();

		for kv in self.iter() {
			match kv {
				(k, v) => {
					k.lua_push(state);
					v.lua_push(state);

					state.raw_set(-3);
				}
			};
		}
	}
}

impl<K: LuaTo + Hash + Eq, V: LuaTo> LuaTo for HashMap<K, V> {
	fn lua_to(state: &state::State, index: int) -> HashMap<K, V> {
		let mut m: HashMap<K, V> = HashMap::new();

		state.push_nil();
		while state.next(index - 1) {
			let k: K = LuaTo::lua_to(state, -2);
			let v: V = LuaTo::lua_to(state, -1);
			state.pop(1);
			m.swap(k, v);
		}
		return m;
	}
}

impl LuaPush for LuaCallback {
	fn lua_push(&self, state: &state::State) {
		state.push_function(*self);
	}
}

pub fn print_stack(state: &state::State) {
	let top = state.get_top();
	if top == 0 { println("stack is empty"); return; }

	printf!("Stack, top: %d\n", top);
	for i in range(1, top + 1) {
		printf!("%d - %s\n", i, state.index_str(i))
	}
}
