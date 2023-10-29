package com.github.markusjx.example;

import com.github.markusjx.generated.NativeStruct;
import com.github.markusjx.generated.RustStruct;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

public class TestNativeStruct {
    @Test
    public void testGetRustStructValue() {
        RustStruct rs = new RustStruct("test");
        assertEquals("test", NativeStruct.getRustStructValue(rs));
    }

    @Test
    public void testGetRustStructValueNull() {
        var msg = assertThrows(NullPointerException.class,
                () -> NativeStruct.getRustStructValue(null)).getMessage();
        assertEquals("The pointer is null", msg);
    }

    @Test
    public void testSetRustStructValue() {
        RustStruct rs = new RustStruct("test");
        NativeStruct.setRustStructValue(rs, "test2");
        assertEquals("test2", rs.getValue());
    }

    @Test
    public void testSetRustStructValueNull() {
        var msg = assertThrows(NullPointerException.class,
                () -> NativeStruct.setRustStructValue(null, "test")).getMessage();
        assertEquals("The pointer is null", msg);

        msg = assertThrows(RuntimeException.class,
                () -> NativeStruct.setRustStructValue(new RustStruct("test"),
                        null)).getMessage();
        assertEquals("Null pointer in get_object_class", msg);
    }

    @Test
    public void testGetRustStructValueOpt() {
        RustStruct rs = new RustStruct("test");
        assertEquals("test", NativeStruct.getRustStructValueOpt(rs));

        assertNull(NativeStruct.getRustStructValueOpt(null));
    }
}
