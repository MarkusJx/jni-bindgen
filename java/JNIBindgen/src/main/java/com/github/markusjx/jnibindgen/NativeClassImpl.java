package com.github.markusjx.jnibindgen;

public interface NativeClassImpl<T extends NativeClass> {
    T getInner();

    default boolean isValid() {
        return getInner().isValid();
    }

    default void destroyNative() {
        getInner().destroyNative();
    }
}
