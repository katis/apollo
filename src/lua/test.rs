use std::hashmap::HashMap;
use std::libc::{c_int};
use lua::{LuaTo,LuaPush};
mod macros;
mod lua;
mod ffi;

#[test]
fn test_noret() {
	lua_fn!( noret(a: int, b: float) )

	let lua = lua::New();
	lua.state().do_str("
		function noret(a, b)
			a = b
		end
	");
	noret(12, 48.23, lua);
	assert!(lua.state().get_top() == 0);
}

#[test]
fn test_add() {
	lua_fn!( add(a: int, b: int) -> int )

	let lua = lua::New();
	lua.state().do_str("
		function add(a, b)
			return a + b
		end
	");
	assert!(add(20, 15, lua) == 35);
	assert!(lua.state().get_top() == 0);
}

#[test]
fn test_concat() {
	lua_fn!( concat(a: &str, b: &str) -> ~str )

	let lua = lua::New();
	lua.state().do_str("
		function concat(a, b)
			return a .. b
		end
	");
	assert!(concat("foo", "bar", lua) == ~"foobar");
	assert!(lua.state().get_top() == 0);
}

#[test]
fn test_reverseplus() {
	lua_fn!( reverseplus(a: ~[float], b: int) -> ~[float] )

	let lua = lua::New();
	lua.state().open_libs();

	lua.state().do_str("
		function reverseplus(arr, p)
			local newArr = {}
			local len = table.getn(arr)

			for i, v in ipairs(arr) do
				newArr[len - i] = v + p
			end

			return newArr
		end
	");

	let result = ~[30.9, 15.2, 20.5];
	let ret = reverseplus(~[10.5, 5.2, 20.9], 10, lua);
	assert!(result.len() == ret.len());
	assert!(lua.state().get_top() == 0);

	let mut i = 0;
	while i < ret.len() {
		assert!(ret[i] == result[i]);
		i += 1
	}
}

#[test]
fn test_swapper() {
	lua_fn!( swapper(m: HashMap<~str, float>) -> HashMap<float, ~str> )

	let lua = lua::New();
	lua.state().open_libs();

	lua.state().do_str("
		function swapper(arr)
			local newArr = {}

			for k, v in pairs(arr) do
				newArr[v] = k
			end

			return newArr
		end
	");

	let mut m: HashMap<~str, float> = HashMap::new();
	m.swap(~"foi", 1.01);
	m.swap(~"tats", 74.75);
	m.swap(~"sis", 51.5);

	let ret = swapper(m, lua);
	assert!(lua.state().get_top() == 0);
	assert!(*ret.get(&1.01) == ~"foi");
	assert!(*ret.get(&74.75) == ~"tats");
	assert!(*ret.get(&51.5) == ~"sis");
}

#[test]
fn test_lua_struct() {
	lua_struct!(
		Foo:
			bar: ~str,
			qwe: int
	);

	let lua = lua::New();

	let foo = Foo::Foo{
		bar: ~"barbarbar",
		qwe: 1234,
	};
	lua.push(foo);

	let foo2: Foo::Foo = lua.pop();
	assert!(lua.state().get_top() == 0);

	assert!(foo2.bar == ~"barbarbar");
	assert!(foo2.qwe == 1234);
}

#[test]
fn test_lua_closure() {
	let lua = lua::New();
	lua.state().do_str("
		function concat(a, b)
			return a .. b
		end

		function add(a, b)
			return a + b
		end
	");
	
	let s: ~[int] = ~[30, 10, 20];
	assert!(s.iter().fold(0, lua_fn!(lua.add |a: int, x: &int| -> int)) == 60);
	assert!(lua.state().get_top() == 0);

	let concat = lua_fn!(lua.concat |a: &str, b: &str| -> ~str);
	let foobar = concat("foo", "bar");
	assert!(lua.state().get_top() == 0);
	assert!(foobar == ~"foobar");
}

#[test]
fn test_lua_table_iter() {
	let lua = lua::New();

	let mut m: HashMap<~str, float> = HashMap::new();
	m.swap(~"foi", 1.01);
	m.swap(~"tats", 74.75);
	m.swap(~"sis", 51.5);

	lua.push(m.clone());

	for kv in lua.table_iter::<~str, float>(-1) {
		match kv {
			(k, v) => {
				assert!(*m.get(&k) == v);
			}
		};
	}
	lua.state().pop(1);
}

#[test]
fn test_lua_array_iter() {
	let lua = lua::New();

	let vect = ~[~"qwe", ~"ads", ~"zxc"];

	lua.push(vect.clone());

	let mut i = 0;
	for v in lua.arr_iter::<~str>(-1) {
		assert!(vect[i] == v);
		i += 1;
	}
	lua.state().pop(1);
}

#[test]
fn test_push_function() {
	let lua = lua::New();

	extern "C" fn add(raw_state: *ffi::lua_State) -> c_int {
		do lua::with_state(raw_state) |state| {
			let argn = state.get_top();
			let mut sum = 0.0;
			let mut i = 1;
			while i <= argn {
				sum += state.to_float(i);
				i += 1;
			}
			state.push_float(sum);
		}
		return 1;
	}

	lua.state().do_str("
	function callFunc(f)
		return f(15.5, 16.6, 18.2)
	end
	");

	lua_fn!( callFunc(cb: lua::LuaCallback) -> float );
	let result = callFunc(add, lua);

	assert!(result == (15.5 + 16.6 + 18.2));
	assert!(lua.state().get_top() == 0);
}

#[test]
fn test_lua_cb() {
	let lua = lua::New();

	lua_cb!(add(a: float, b: float, c: float) -> float {
		a + b + c
	}); 

	lua.state().do_str("
	function callFunc(f)
		return f(15.5, 16.6, 18.2)
	end
	");

	lua_fn!( callFunc(cb: lua::LuaCallback) -> float );
	let result = callFunc(add, lua);

	assert!(result == (15.5 + 16.6 + 18.2));
	assert!(lua.state().get_top() == 0);
}
