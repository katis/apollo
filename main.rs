// lua_fn!(funcname params!(foo: int, bar:int, suffix: string) -> int)
use lua::{LuaPush, LuaPop};
mod lua;

macro_rules! lua_fn(
	($func:ident($( $arg:ident: $argty:ty ),* ) -> Result<$rty:ty, LuaErr>) => (
		fn $func ( $( $arg: $argty, )* _lua: &lua::Lua ) -> Result<$rty, lua::LuaErr> {
			_lua.get_global(stringify!($func));
			let mut _len = 0;

			$(
				$arg.lua_push(_lua);
				_len += 1; 
			 )*

			return match _lua.pcall(_len, 1, 0) {
				Some(_err) => Err(_err),
				None => Ok(LuaPop::lua_pop(_lua))
			}
		}
	);
	($func:ident($( $arg:ident: $argty:ty ),* ) -> Option<LuaErr>) => (
		fn $func ( $( $arg: $argty, )* _lua: &lua::Lua ) -> Option<lua::LuaErr> {
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
	reverse(a: ~[float]) -> Result<~[float], LuaErr>
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

	match reverse(~[12.45, 45.12, 12.1, 69.69], lua) {
		Ok(vect) => {
			println(fmt!("%?", vect));
		},
		Err(err) => { println(err.to_str()); return; }
	}
}
