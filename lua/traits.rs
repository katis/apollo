pub use self::lua_state::*;
use std::hashmap::HashMap;
use std::vec::OwnedVector;
mod lua_state;

pub trait LuaPush {
	fn lua_push(&self, lua: &LuaState);
}

pub trait LuaTo {
	fn lua_to(lua: &LuaState, index: int) -> Self;
}

impl LuaPush for float {
	fn lua_push(&self, lua: &LuaState) {
		lua.push_float(*self);
	}
}

impl LuaTo for float {
	fn lua_to(lua: &LuaState, index: int) -> float {
		return lua.to_float(index);
	}
}

impl LuaPush for int {
	fn lua_push(&self, lua: &LuaState) {
		lua.push_int(*self);
	}
}

impl LuaTo for int {
	fn lua_to(lua: &LuaState, index: int) -> int {
		return lua.to_int(index);
	}
}

impl LuaPush for ~str {
	fn lua_push(&self, lua: &LuaState) {
		lua.push_str(*self);
	}
}

impl<'self> LuaPush for &'self str {
	fn lua_push(&self, lua: &LuaState) {
		lua.push_str(*self);
	}
}

impl LuaTo for ~str {
	fn lua_to(lua: &LuaState, index: int) -> ~str {
		lua.to_str(index)
	}
}

impl<T: LuaPush> LuaPush for ~[T] {
	fn lua_push(&self, lua: &LuaState) {
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
	fn lua_to(lua: &LuaState, index: int) -> ~[T] {
		let mut vect = ~[];

		lua.push_nil();
		if index != -1 { lua.insert(index); }
		while lua.next(index - 1) {
			vect.push( LuaTo::lua_to(lua, index) );
			lua.remove(index);
		}

		return vect;
	}
}

impl<'self, K: LuaPush + Hash + Eq, V: LuaPush> LuaPush for &'self HashMap<K, V> {
	fn lua_push(&self, lua: &LuaState) {
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
	fn lua_push(&self, lua: &LuaState) {
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
	fn lua_to(lua: &LuaState, index: int) -> HashMap<K, V> {
		let mut m: HashMap<K, V> = HashMap::new();

		lua.push_nil();
		if index != -1 { lua.insert(index); }
		while lua.next(index - 1) {
			let v: V = LuaTo::lua_to(lua, index);
			lua.remove(index);

			let k: K = LuaTo::lua_to(lua, index);
			m.swap(k, v);
		}
		return m;
	}
}

pub fn print_stack(lua: &LuaState) {
	let top = lua.get_top();
	if top == 0 { println("stack is empty"); return; }

	println("");
	for i in range(1, top + 1) {
		printf!("%d - %s\n", i, lua.index_str(i))
	}
}
