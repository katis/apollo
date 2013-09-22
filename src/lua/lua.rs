pub use self::state::*;
use std::hashmap::HashMap;
mod state;

struct Lua {
	state: ~state::State
}

pub fn New() -> ~Lua {
	~Lua {
		state: state::NewState()
	}
}

impl Lua {
	pub fn push<T: LuaPush>(&self, p: T) {
		p.lua_push(self.state);
	}

	pub fn i_to<T: LuaTo>(&self, index: int) -> T {
		LuaTo::lua_to(self.state, index)
	}

	pub fn pop<T: LuaTo>(&self) -> T {
		let v: T = self.i_to(-1);
		self.state.pop(1);
		return v;
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

pub fn print_stack(state: &state::State) {
	let top = state.get_top();
	if top == 0 { println("stack is empty"); return; }

	printf!("Stack, top: %d\n", top);
	for i in range(1, top + 1) {
		printf!("%d - %s\n", i, state.index_str(i))
	}
}
