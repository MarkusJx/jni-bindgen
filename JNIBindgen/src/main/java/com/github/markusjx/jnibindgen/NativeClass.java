package com.github.markusjx.jnibindgen;

public abstract class NativeClass {
    protected volatile long ptr;
    private final NativeDrop drop;

    protected NativeClass(long ptr, Object referent) {
        this.ptr = ptr;
        drop = new NativeDrop(referent, this::dropNative);
    }

    protected synchronized void dropNative() {
        if (isValid()) {
            this.destruct();
            this.ptr = 0;
        }
    }

    protected abstract void destruct();

    public synchronized boolean isValid() {
        return this.ptr != 0;
    }

    public synchronized long getPtr() {
        return this.ptr;
    }

    public void destroyNative() {
        if (!isValid()) {
            throw new IllegalStateException("Native object already destroyed");
        }

        drop.destruct();
    }
}
