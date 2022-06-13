extern crate flamboyant;

fn main() {
    unsafe {
        flamboyant::core::rb_flamboyant_serve(
            flamboyant::ruby_ext::Nil.into(),
            flamboyant::ruby_ext::Nil.into(),
        );
    }
}
