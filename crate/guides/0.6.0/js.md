# Interaction with Javascript

## Calling Javascript functions
If you have Javascript functions in your app, you can call them from seed code using 
`wasm_bindgen`. For a detailed
example, see the [official example](https://github.com/seed-rs/seed/tree/master/examples/update_from_js).

For example, you might have a Javascript function defined elsewhere in your document,
like this:
```html
<script type="text/javascript">
    function addOne(val) {
        return val + 1
    }
</script>

<section id="app"></section>
<script type="module">
    import init from '/static/pkg/mypage.js';
    init('/static/pkg/mypage.wasm');
</script>
```


Define a function like this in your app, where `addOne` here is the same name as the
javascript function you wish to call.
```rust
/// Allows calling the JS function getCookie, for CSRF tokens.
#[wasm_bindgen]
extern "C" {
    fn addOne(val: &i32) -> i32;
}
```

You can then call this anywhere in your app, eg:
```rust
h1![ format!("Two plus one equals {}", addOne(2)) ]
```

An example of how you might initialize a Seed app, populating a model from Javascript:
```rust
fn after_mount(_: Url, orders: &mut impl Orders<Msg>) -> AfterMount<Model> {
    AfterMount::new(getJsData())
}

#[wasm_bindgen]
extern "C" {
    fn getJsData() -> Model;
}
```
Here, we assume `getJsData()` is defined before loading Seed, and that it
passes data that can be deserialized into the model.
Note that you can also pass arguments to JS functions like this.


For more info, reference [this wasm-bindgen example](https://rustwasm.github.io/docs/wasm-bindgen/examples/import-js.html).
