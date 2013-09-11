extern mod extra;
use std::hashmap::HashMap;
use lua::{LuaPush, LuaTo};
mod lua;

macro_rules! lua_fn(
	($func:ident($( $arg:ident: $argty:ty ),* ) -> Result<$rty:ty, LuaErr>) => (
		fn $func ( $( $arg: $argty, )* _lua: &lua::LuaState ) -> Result<$rty, lua::LuaErr> {
			_lua.get_global(stringify!($func));
			match _lua.index_type(_lua.get_top()) {
				lua::Function => {},
				_ => { return Err(lua::Syntax(fmt!("unknown function %s", stringify!($func)))); }
			}

			let mut _len = 0;

			$(
				$arg.lua_push(_lua);
				_len += 1; 
			 )*

			let mut _ret: Result<$rty, lua::LuaErr>;
			match _lua.pcall(_len, 1, 0) {
				Some(_err) => { _ret = Err(_err); },
				None => { _ret = Ok(LuaTo::lua_to(_lua, -1)); _lua.pop(1); }
			};
			return _ret;
		}
	);
	($func:ident($( $arg:ident: $argty:ty ),* ) -> Option<LuaErr>) => (
		fn $func ( $( $arg: $argty, )* _lua: &lua::LuaState ) -> Option<lua::LuaErr> {
			_lua.get_global(stringify!($func));
			let mut _len = 0;

			$(
				$arg.lua_push(_lua);
				_len += 1;
			)*

			return _lua.pcall(_len, 0, 0);
		}
	);
)

lua_fn!(
	add(a: int, b: int) -> Result<float, LuaErr>
)

lua_fn!(
	noret(a: int, b: float) -> Option<LuaErr>
)

lua_fn!(
	concat(a: &str, b: &str) -> Result<~str, LuaErr>
)

lua_fn!(
	reverseplus(a: ~[float], b: int) -> Result<~[float], LuaErr>
)

lua_fn!(
	swapper(m: HashMap<~str, float>) -> Result<HashMap<float, ~str>, LuaErr>
)

fn main() {
	let lua = lua::New();

	match lua.do_file("sample.lua") {
		Some(err) => { println(err.to_str()); return; },
		_ => {}
	};

	match add(10, 25, lua) {
		Ok(x) => {println(fmt!("%f", x));},
		Err(err) => {println(err.to_str()); return;}
	};


	match noret(12, 42.78, lua) {
		Some(err) => { println(err.to_str()); return; },
		_ => {}
	}


	match concat("foo ", "bar", lua) {
		Ok(s) => { println(s); },
		Err(err) => { println(err.to_str()); return; }
	}

	match reverseplus(~[12.45, 45.12, 12.1, 69.69], 10, lua) {
		Ok(vect) => {
			println(fmt!("%?", vect));
		},
		Err(err) => { printf!("reverseplus error: %s\n", err.to_str()); return; }
	}

	let mut m: HashMap<~str, float> = HashMap::new();
	m.swap(~"foi", 1.01);
	m.swap(~"tats", 74.75);
	m.swap(~"sis", 51.5);

	match swapper(m, lua) {
		Ok(newm) => {
			for kv in newm.iter() {
				match(kv) {
					(k, v) => {
						printf!("%?, %?\n", *k, *v);
					}
				}
			}
		},
		Err(err) => { printf!("swap error: %s\n", err.to_str()); return; }
	}

	lua::print_stack(lua);
}
