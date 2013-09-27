#[macro_escape];

macro_rules! lua_struct(
	($s:ident: $( $field:ident: $fty:ty ),+ ) => (
		mod $s {
			use lua::*;
			pub struct $s {
				$( $field: $fty, )+
			}

			impl LuaPush for $s {
				fn lua_push(&self, state: &State) {
					state.new_table();

					$(
					state.push_str(stringify!($field));
					self.$field.lua_push(state);
					state.raw_set(-3);
					)+
				}
			}

			impl LuaTo for $s {
				fn lua_to(state: &State, index: int) -> $s {
					$s {
						$(
						$field: {
							state.get_field(index, stringify!($field));
							let r = LuaTo::lua_to(state, index);
							state.pop(1);
							r
						},
						)+
					}
				}
			}
		}
	);
)

macro_rules! lua_fn(
	// function with a return value
	($func:ident($( $arg:ident: $argty:ty ),* ) -> $rty:ty) => (
		fn $func ( $( $arg: $argty, )* _lua: &lua::Lua ) -> $rty {
			_lua.state().get_global(stringify!($func));
			match _lua.state().index_type(_lua.state().get_top()) {
				lua::TFunction => {},
				_ => { fail!(fmt!("lua_fn error: unknown function %s", stringify!($func))); }
			}

			let mut _len = 0;

			$(
				_lua.push($arg);
				_len += 1; 
			 )*

			_lua.state().pcall(_len, 1, 0);

			let _ret: $rty = _lua.pop();
			return _ret;
		}
	);
	// function with a return value
	( $($table:ident)::+ . $func:ident($( $arg:ident: $argty:ty ),* ) -> $rty:ty) => (
		fn $func ( $( $arg: $argty, )* _lua: &lua::Lua ) -> $rty {
			let mut _i = lua::LUA_GLOBALSINDEX as int;
			let _top = _lua.state().get_top();

			$(
				match _lua.state().index_type(_i) {
					lua::TTable => {},
					_ => { fail!(fmt!("lua_fn error: table %s not found", stringify!($table))); }
				}
				_lua.state().get_field(_i, stringify!($table));
				_i = -1;
			)+

			_lua.state().get_field(_i, stringify!($func));
			match _lua.state().index_type(-1) {
				lua::TFunction => {},
				_ => {
					fail!(fmt!("lua_fn error: unknown function %s",
						$( stringify!($table) + "." + )+ stringify!($func)));
				}
			}

			let mut _len = 0;

			$(
				_lua.push($arg);
				_len += 1; 
			 )*

			_lua.state().pcall(_len, 1, 0);

			let _ret: $rty = _lua.pop();
			_lua.state().set_top(_top);
			return _ret;
		}
	);
	// function with no return value
	($func:ident($( $arg:ident: $argty:ty ),* ) ) => (
		fn $func ( $( $arg: $argty, )* _lua: &lua::Lua ) {
			_lua.state().get_global(stringify!($func));
			let mut _len = 0;

			$(
				_lua.push($arg);
				_len += 1;
			)*

			_lua.state().pcall(_len, 0, 0);
		}
	);
	// Closure with return value
	($lua:ident.$func:ident | $( $arg:ident: $argty:ty),* | -> $rty:ty) => (
		| $($arg: $argty),* | {
			$lua.state().get_global(stringify!($func));
			match $lua.state().index_type($lua.state().get_top()) {
				lua::TFunction => {},
				_ => { fail!(fmt!("lua_fn error: unknown function %s", stringify!($func))); }
			}

			let mut _len = 0;

			$(
				$lua.push($arg);
				_len += 1; 
			 )*

			$lua.state().pcall(_len, 1, 0);

			let _ret: $rty = $lua.pop();
			_ret
		}
	);
)

macro_rules! lua_cb(
	($func:ident ( $($arg:ident: $aty:ty ),* ) -> $rty:ty
		$bl:block
	) => (
		extern "C" fn $func(_raw_state: *ffi::lua_State) -> c_int {
			do lua::with_state(_raw_state) |state| {
				let mut _i = 1;
				$(
					let $arg: $aty = LuaTo::lua_to(state, _i); _i += 1;
				)*
				let _ret: $rty = {if true $bl else { fail!("lua_cb! impossible") } };
				_ret.lua_push(state);
			}
			return 1;
		}
	);
)
