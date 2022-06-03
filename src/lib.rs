pub mod core;
pub mod ruby_ext;

use std::error::Error;

use coffret::*;

// #[cfg(test)]
// mod tests {

//     #[test]
//     fn it_works() {
//         let result = 2 + 2;
//         assert_eq!(result, 4);
//     }
// }

fn init_flamboyant_internal() -> Result<(), Box<dyn Error>> {
    println!("Rust loaded");
    let object = class::object_class();
    let klass = class::define_class("Flamboyant", object);

    //unsafe { rb_define_singleton_method(klass, function_name.as_ptr(), Some(test_hola), 0) }

    //let callback = class::make_callback(&crate::core::rb_flamboyant_serve);
    let callback: crate::ruby_ext::RubyFn =
        (crate::core::rb_flamboyant_serve as unsafe extern "C" fn(u64, u64) -> u64).into();

    class::define_method(klass, "serve", callback.into(), 1);
    Ok(())
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Init_flamboyant() {
    match init_flamboyant_internal() {
        Err(e) => exception::rustly_raise(e.as_ref()),
        Ok(_) => {}
    }
}
