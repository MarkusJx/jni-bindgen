package com.github.markusjx.jnibindgen;

public class NativeDrop extends DestructorThread.Destructor {
    private final Runnable destructor;

    public NativeDrop(Object referent, Runnable destructor) {
        super(referent);
        this.destructor = destructor;
    }

    @Override
    protected synchronized void destruct() {
        this.destructor.run();
    }
}
