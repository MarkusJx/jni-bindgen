use jni::JNIEnv;
use jni_bindgen::objects::traits::{FromJNI, ObjectFromJNI};

#[jni(namespace = "com.github.markusjx.generated")]
pub trait TestTrait {
    /// Test
    fn test(&self, env: &mut JNIEnv, val: String) -> jni_bindgen::Result<String>;
}

struct StructUsingTrait;

#[jni(namespace = "com.github.markusjx.generated", load_lib = "example_lib")]
impl StructUsingTrait {
    #[jni]
    fn use_trait<'a>(
        #[jni(class_name = "TestTrait")] trait_obj: Box<dyn TestTrait + 'a>,
        env: &mut JNIEnv<'a>,
    ) -> jni_bindgen::Result<String> {
        trait_obj.test(env, "test".to_string())
    }
}
