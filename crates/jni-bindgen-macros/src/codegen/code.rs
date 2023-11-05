use std::collections::HashSet;
use std::fmt::Display;

const DESTRUCT: &str = r#"
    @Override
    protected void destruct() {
        drop(this.ptr);
    }
"#;

pub fn interface(
    namespace: &str,
    class_name: &str,
    comment: String,
    methods: String,
    additional_imports: HashSet<String>,
) -> String {
    format_code(format!(
        r#"
    package {namespace};
    
    {}
    
    {comment}
    public interface {class_name} {{
        {methods}
    }}
    "#,
        additional_imports
            .into_iter()
            .map(|i| format!("import {i};"))
            .collect::<Vec<String>>()
            .join("\n"),
    ))
}

pub fn outer_class(
    namespace: &str,
    class_name: &str,
    comment: String,
    methods: String,
    mut constructors: String,
    inner: String,
    mut additional_imports: HashSet<String>,
) -> String {
    let mut inner_ty = "".to_string();
    if !constructors.is_empty() {
        inner_ty = format!("private final {class_name}Native inner;");
        additional_imports.insert("com.github.markusjx.jnibindgen.NativeClass".to_string());
        additional_imports.insert("com.github.markusjx.jnibindgen.NativeClassImpl".to_string());
    }

    let mut implements = "".to_string();
    if !constructors.is_empty() {
        implements = format!(
            " implements NativeClassImpl<{class_name}.{class_name}Native>",
            class_name = class_name
        );
    }

    let mut get_inner = "".to_string();
    if !constructors.is_empty() {
        get_inner = format!(
            r#"
    @Override
    public {class_name}Native getInner() {{
        return inner;
    }}"#
        );
    } else {
        constructors = disable_ctor(class_name);
    }

    format_code(format!(
        r#"package {};

        {}
        
        {comment}
        public class {class_name} {implements}{{
    {inner_ty}
    
    {constructors}

    {methods}
    
    /**
     * Get the type of the underlying native class
     * in a hashed form. This is used to check if
     * the native class is the same as the one
     * that is expected.
     *
     * @return The type hash of this class
     */
    public static long getTypeHash() {{
        return {class_name}Native.getTypeHash();
    }}

    {get_inner}

    {inner}
}}"#,
        namespace,
        additional_imports
            .into_iter()
            .map(|i| format!("import {i};"))
            .collect::<Vec<String>>()
            .join("\n"),
    ))
}

pub fn inner_class(
    class_name: &str,
    methods: String,
    mut constructors: String,
    load_lib: Option<String>,
) -> String {
    let init_lib = if let Some(init) = load_lib {
        load_library(&init)
    } else {
        String::new()
    };

    let mut extends = "".to_string();
    if !constructors.is_empty() {
        extends = "extends NativeClass ".to_string();
    }

    let mut destruct = "".to_string();
    if !constructors.is_empty() {
        destruct = DESTRUCT.to_string();
    } else {
        constructors = disable_ctor(class_name.to_string() + "Native");
    }

    format!(
        r#"
        public static class {class_name}Native {extends}{{
            {init_lib}
            {constructors}
            {methods}
            {destruct}
        }}
    "#
    )
}

fn disable_ctor<T: Display>(class_name: T) -> String {
    format!(
        r#"
    /**
     * Disable instantiation of {class_name}
     */
    private {class_name}() {{
        throw new UnsupportedOperationException("{class_name} cannot be instantiated");
    }}
    "#
    )
}

fn load_library(lib_name: &str) -> String {
    format!(
        r#"        static {{
            System.loadLibrary("{lib_name}");
        }}"#
    )
}

fn format_code(code: String) -> String {
    let mut indent = 0;
    let split = code
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();

    split
        .iter()
        .enumerate()
        .map(|(i, line)| {
            if line.starts_with('}') {
                indent -= 1;
            }

            let next = split.get(i + 1).map(|s| s.to_string()).unwrap_or_default();
            let last = if i > 0 {
                split.get(i - 1).map(|s| s.to_string()).unwrap_or_default()
            } else {
                "".to_string()
            };

            let res = format!(
                "{pre_newline}{indent}{line}{newline}",
                pre_newline = if (line.starts_with("public class") && !last.contains('*'))
                    || (line.starts_with("/**") && last.starts_with("import"))
                {
                    "\n"
                } else {
                    ""
                },
                indent = if line.starts_with('*') {
                    " ".repeat(indent * 4 + 1)
                } else {
                    " ".repeat(indent * 4)
                },
                newline = if !line.ends_with('{')
                    && !line.contains('*')
                    && (!line.ends_with(';')
                        || line.contains("native")
                        || line.contains("private")
                        || line.contains("package"))
                    && !line.contains('@')
                    && !next.contains('}')
                {
                    "\n"
                } else {
                    ""
                }
            );

            if line.ends_with('{') {
                indent += 1;
            }

            res
        })
        .collect::<Vec<_>>()
        .join("\n")
}
