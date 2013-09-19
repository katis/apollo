extern mod extra;
use std::hashmap::HashMap;
mod lua;
mod macros;

lua_fn!( noret(a: int, b: float) )
lua_fn!( add(a: int, b: int) -> float )
lua_fn!( concat(a: &str, b: &str) -> ~str )
lua_fn!( reverseplus(a: ~[float], b: int) -> ~[float] )
lua_fn!( swapper(m: HashMap<~str, float>) -> HashMap<float, ~str> )

lua_struct!(
	Foo:
		bar: ~str,
		qwe: int
)

fn main() {
	let lua = lua::New();
	lua.state.open_libs();
	lua.state.do_file("sample.lua");

	noret(12, 42.78, lua);
	printf!("%f\n", add(10, 25, lua));
	printf!("%s\n", concat("foo ", "bar", lua));
	printf!("%?\n", reverseplus(~[12.45, 45.12, 12.1, 69.69], 10, lua));

	let mut m: HashMap<~str, float> = HashMap::new();
	m.swap(~"foi", 1.01);
	m.swap(~"tats", 74.75);
	m.swap(~"sis", 51.5);

	let newm = swapper(m, lua);
	print("{ ");
	for kv in newm.iter() {
		match(kv) {
			(k, v) => {
				printf!("[%?: %?] ", *k, *v);
			}
		}
	}
	print("}\n");

	let foo = Foo::Foo{
		bar: ~"barbarbar",
		qwe: 1234,
	};
	lua.push(foo);

	let foo2: Foo::Foo = lua.pop();
	printf!("%?\n", foo2);

}
