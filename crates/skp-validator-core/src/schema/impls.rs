use super::{TypeValidation, ValidationMetadata};

// Primitive types have no validation by default
macro_rules! impl_empty_metadata {
    ($($t:ty),*) => {
        $(
            impl ValidationMetadata for $t {
                fn get_validation_rules() -> TypeValidation {
                    TypeValidation::default()
                }
            }
        )*
    };
}

impl_empty_metadata!(String, bool, char, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);
impl_empty_metadata!(&str);

// Option delegates to T's metadata
impl<T: ValidationMetadata> ValidationMetadata for Option<T> {
    fn get_validation_rules() -> TypeValidation {
        T::get_validation_rules()
    }
}

// Vec delegates to T's metadata
impl<T: ValidationMetadata> ValidationMetadata for Vec<T> {
    fn get_validation_rules() -> TypeValidation {
        T::get_validation_rules()
    }
}

// Smart pointers delegate
impl<T: ValidationMetadata> ValidationMetadata for Box<T> {
    fn get_validation_rules() -> TypeValidation {
        T::get_validation_rules()
    }
}

impl<T: ValidationMetadata> ValidationMetadata for std::rc::Rc<T> {
    fn get_validation_rules() -> TypeValidation {
        T::get_validation_rules()
    }
}

impl<T: ValidationMetadata> ValidationMetadata for std::sync::Arc<T> {
    fn get_validation_rules() -> TypeValidation {
        T::get_validation_rules()
    }
}

// Collections that hold values
impl<K, V: ValidationMetadata> ValidationMetadata for std::collections::HashMap<K, V> {
    fn get_validation_rules() -> TypeValidation {
        V::get_validation_rules()
    }
}

impl<K, V: ValidationMetadata> ValidationMetadata for std::collections::BTreeMap<K, V> {
    fn get_validation_rules() -> TypeValidation {
        V::get_validation_rules()
    }
}

impl<T: ValidationMetadata> ValidationMetadata for std::collections::HashSet<T> {
    fn get_validation_rules() -> TypeValidation {
        T::get_validation_rules()
    }
}

impl<T: ValidationMetadata> ValidationMetadata for std::collections::BTreeSet<T> {
    fn get_validation_rules() -> TypeValidation {
        T::get_validation_rules()
    }
}
