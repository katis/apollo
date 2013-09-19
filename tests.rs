use std::hashmap::HashMap;
mod macros;
mod lua;

lua_fn!( noret(a: int, b: float) )

#[test]
fn test_noret() {
	let lua = lua::New();
	lua.state.do_file("sample.lua");
	noret(12, 48.23, lua);
}

lua_fn!( add(a: int, b: int) -> int )

#[test]
fn test_add() {
	let lua = lua::New();
	lua.state.do_file("sample.lua");
	assert!(add(20, 15, lua) == 35);
}

lua_fn!( concat(a: &str, b: &str) -> ~str )

#[test]
fn test_concat() {
	let lua = lua::New();
	lua.state.do_file("sample.lua");
	assert!(concat("foo", "bar", lua) == ~"foobar");
}

lua_fn!( reverseplus(a: ~[float], b: int) -> ~[float] )

#[test]
fn test_reverseplus() {
	let lua = lua::New();
	lua.state.open_libs();
	lua.state.do_file("sample.lua");

	let result = ~[30.9, 15.2, 20.5];
	let ret = reverseplus(~[10.5, 5.2, 20.9], 10, lua);
	assert!(result.len() == ret.len());

	let mut i = 0;
	while i < ret.len() {
		assert!(ret[i] == result[i]);
		i += 1
	}
}

lua_fn!( swapper(m: HashMap<~str, float>) -> HashMap<float, ~str> )

#[test]
fn test_swapper() {
	let lua = lua::New();
	lua.state.open_libs();
	lua.state.do_file("sample.lua");

	let mut m: HashMap<~str, float> = HashMap::new();
	m.swap(~"foi", 1.01);
	m.swap(~"tats", 74.75);
	m.swap(~"sis", 51.5);

	let ret = swapper(m, lua);
	assert!(*ret.get(&1.01) == ~"foi");
	assert!(*ret.get(&74.75) == ~"tats");
	assert!(*ret.get(&51.5) == ~"sis");
}

lua_struct!(
	Foo:
		bar: ~str,
		qwe: int
)

#[test]
fn test_lua_struct() {
	let lua = lua::New();
	lua.state.open_libs();
	lua.state.do_file("sample.lua");

	let foo = Foo::Foo{
		bar: ~"barbarbar",
		qwe: 1234,
	};
	lua.push(foo);

	let foo2: Foo::Foo = lua.pop();

	assert!(foo2.bar == ~"barbarbar");
	assert!(foo2.qwe == 1234);
}
