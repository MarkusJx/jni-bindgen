package com.github.markusjx.example;

import static org.junit.jupiter.api.Assertions.*;

import com.github.markusjx.generated.RustStruct;
import com.github.markusjx.jnibindgen.NativeExecutionException;
import org.junit.jupiter.api.Test;

public class TestRustStruct {
    @Test
    public void testCreate() {
        RustStruct rs = new RustStruct("test");
        assertEquals("test", rs.getValue());
    }

    @Test
    public void testGetValue() {
        RustStruct rs = new RustStruct("test");
        assertEquals("test", rs.getValue());
    }

    @Test
    public void testSetValue() {
        RustStruct rs = new RustStruct("test");
        rs.setValue("test2");
        assertEquals("test2", rs.getValue());
    }

    @Test
    public void testNullableString() {
        assertNull(RustStruct.getString(null));
        assertEquals("test", RustStruct.getString("test"));
    }

    @Test
    public void testNullableInt() {
        assertNull(RustStruct.getInt(null));
        assertEquals(5, RustStruct.getInt(5));
    }

    @Test
    public void testNullableLong() {
        assertNull(RustStruct.getLong(null));
        assertEquals(5, RustStruct.getLong(5L));
    }

    @Test
    public void testNullableFloat() {
        assertNull(RustStruct.getFloat(null));
        assertEquals(5, RustStruct.getFloat(5f));
    }

    @Test
    public void testNullableDouble() {
        assertNull(RustStruct.getDouble(null));
        assertEquals(5, RustStruct.getDouble(5d));
    }

    @Test
    public void testNullableBool() {
        assertNull(RustStruct.getBool(null));
        assertEquals(true, RustStruct.getBool(true));
    }

    @Test
    public void testNullableChar() {
        assertNull(RustStruct.getChar(null));
        assertEquals('a', RustStruct.getChar('a'));
    }

    @Test
    public void testNullableByte() {
        assertNull(RustStruct.getByte(null));
        assertEquals((byte) 5, RustStruct.getByte((byte) 5));
    }

    @Test
    public void testNullableShort() {
        assertNull(RustStruct.getShort(null));
        assertEquals((short) 5, RustStruct.getShort((short) 5));
    }

    @Test
    public void testThrow() {
        var msg =
                assertThrows(NativeExecutionException.class, () -> RustStruct.throwError("test"))
                        .getMessage();
        assertEquals("test", msg);
    }

    @Test
    public void testThrowOther() throws Exception {
        var msg =
                assertThrows(
                                InterruptedException.class,
                                () ->
                                        RustStruct.throwOtherError(
                                                InterruptedException.class.getName(), "test"))
                        .getMessage();
        assertEquals("test", msg);
    }
}
