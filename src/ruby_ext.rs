use rb_sys::*;

#[repr(C)]
pub struct WrappedRubyValue {
    value: u32,
    padding: u32,
}

impl From<WrappedRubyValue> for RubyValue {
    fn from(from: WrappedRubyValue) -> Self {
        let v = unsafe { std::mem::transmute::<WrappedRubyValue, RubyValue>(from) };
        v
    }
}

#[allow(non_upper_case_globals)]
pub const True: WrappedRubyValue = WrappedRubyValue {
    value: ruby_special_consts::RUBY_Qtrue as u32,
    padding: 0,
};

#[allow(non_upper_case_globals)]
pub const False: WrappedRubyValue = WrappedRubyValue {
    value: ruby_special_consts::RUBY_Qfalse as u32,
    padding: 0,
};

#[allow(non_upper_case_globals)]
pub const Nil: WrappedRubyValue = WrappedRubyValue {
    value: ruby_special_consts::RUBY_Qnil as u32,
    padding: 0,
};

pub struct RubyFn {
    value: unsafe extern "C" fn() -> RubyValue,
}

impl From<RubyFn> for unsafe extern "C" fn() -> RubyValue {
    fn from(from: RubyFn) -> Self {
        from.value
    }
}

impl From<unsafe extern "C" fn() -> RubyValue> for RubyFn {
    fn from(from: unsafe extern "C" fn() -> RubyValue) -> Self {
        RubyFn { value: from }
    }
}

impl From<unsafe extern "C" fn(RubyValue) -> RubyValue> for RubyFn {
    fn from(from: unsafe extern "C" fn(RubyValue) -> RubyValue) -> Self {
        let value = unsafe {
            std::mem::transmute::<
                unsafe extern "C" fn(RubyValue) -> RubyValue,
                unsafe extern "C" fn() -> RubyValue,
            >(from)
        };
        RubyFn { value }
    }
}

impl From<unsafe extern "C" fn(RubyValue, RubyValue) -> RubyValue> for RubyFn {
    fn from(from: unsafe extern "C" fn(RubyValue, RubyValue) -> RubyValue) -> Self {
        let value = unsafe {
            std::mem::transmute::<
                unsafe extern "C" fn(RubyValue, RubyValue) -> RubyValue,
                unsafe extern "C" fn() -> RubyValue,
            >(from)
        };
        RubyFn { value }
    }
}
