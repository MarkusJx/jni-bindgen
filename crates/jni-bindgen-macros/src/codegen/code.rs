pub const DESTRUCT: &str = r#"
    @Override
    protected void destruct() {
        drop(this.ptr);
    }
"#;

pub fn outer_class(
    namespace: &str,
    class_name: &str,
    methods: String,
    constructors: String,
    inner: String,
    throws: bool,
) -> String {
    format!(
        r#"package {};
        
        import com.github.markusjx.rust.NativeClass;
        import com.github.markusjx.rust.NativeClassImpl;{}
        
        public class {class_name} implements NativeClassImpl<{class_name}.{class_name}Native> {{
    private final {class_name}Native inner;

    {methods}
    
    {constructors}

    @Override
    public {class_name}Native getInner() {{
        return inner;
    }}

    {inner}
    }}"#,
        namespace,
        if throws {
            "\nimport com.github.markusjx.rust.NativeExecutionException;"
        } else {
            ""
        },
    )
}

pub fn load_library(lib_name: &str) -> String {
    format!(
        r#"static {{
        System.loadLibrary("{lib_name}");
    }}"#
    )
}
