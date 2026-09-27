pub use crate::avm2::object::dictionary_allocator;

use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::function::FunctionArgs;
use crate::avm2::object::DictionaryObjectWeak;
use crate::avm2::value::Value;
use gc_arena::Gc;

/// skua #13: `Dictionary.setWeakKeys`, called by the constructor for `new Dictionary(true)`.
pub fn set_weak_keys<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dict) = this.as_object().and_then(|o| o.as_dictionary_object()) {
        dict.skua_set_weak_keys();
        activation
            .avm2()
            .skua_weak_dictionaries
            .push(DictionaryObjectWeak(Gc::downgrade(dict.0)));
    }
    Ok(Value::Undefined)
}
