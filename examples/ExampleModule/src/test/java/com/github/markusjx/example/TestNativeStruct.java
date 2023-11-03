package com.github.markusjx.example;

import static org.junit.jupiter.api.Assertions.*;

import com.github.markusjx.generated.NativeStruct;
import com.github.markusjx.generated.RustStruct;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;

public class TestNativeStruct {
    @Test
    public void testGetRustStructValue() {
        RustStruct rs = new RustStruct("test");
        assertEquals("test", NativeStruct.getRustStructValue(rs));
    }

    @Test
    public void testGetRustStructValueNull() {
        var msg =
                assertThrows(
                                NullPointerException.class,
                                () -> NativeStruct.getRustStructValue(null))
                        .getMessage();
        assertEquals("The pointer is null", msg);
    }

    @Test
    public void testGetRustStructValueOpt() {
        RustStruct rs = new RustStruct("test");
        assertEquals("test", NativeStruct.getRustStructValueOpt(rs));

        assertNull(NativeStruct.getRustStructValueOpt(null));
    }

    @Test
    public void testGetObj() {
        RustStruct rs = new RustStruct("test");

        assertEquals("test", NativeStruct.getObj("test"));
        assertEquals(1, NativeStruct.getObj(1));
        assertEquals(1.0, NativeStruct.getObj(1.0));
        assertEquals(true, NativeStruct.getObj(true));
        assertEquals('a', NativeStruct.getObj('a'));
        assertEquals(rs, NativeStruct.getObj(rs));
        assertNull(NativeStruct.getObj(null));
    }

    @Test
    public void testGetVec() {
        var vec = NativeStruct.getVec(List.of("test", "test2"));
        assertEquals(2, vec.size());
        assertEquals("test", vec.get(0));
        assertEquals("test2", vec.get(1));
    }

    @Test
    public void testGetVecNull() {
        var msg =
                assertThrows(NullPointerException.class, () -> NativeStruct.getVec(null))
                        .getMessage();
        assertEquals("Null pointer in get_list obj argument", msg);
    }

    @Test
    public void testGetHashmap() {
        var map = NativeStruct.getHashmap(Map.of("test", "test2"));
        assertEquals(1, map.size());
        assertEquals("test2", map.get("test"));
    }

    @Test
    public void testGetHashmapNull() {
        var msg =
                assertThrows(NullPointerException.class, () -> NativeStruct.getHashmap(null))
                        .getMessage();
        assertEquals("Null pointer in get_map obj argument", msg);
    }

    @Test
    public void testGetVecOpt() {
        var vec = NativeStruct.getVecOpt(List.of("test", "test2"));
        assertEquals(2, vec.size());
        assertEquals("test", vec.get(0));
        assertEquals("test2", vec.get(1));

        assertNull(NativeStruct.getVecOpt(null));
    }

    @Test
    public void testGetHashmapOpt() {
        var map = NativeStruct.getHashmapOpt(Map.of("test", "test2"));
        assertEquals(1, map.size());
        assertEquals("test2", map.get("test"));

        assertNull(NativeStruct.getHashmapOpt(null));
    }

    @Test
    public void testGetVecValues() {
        var vec =
                NativeStruct.getVecValues(List.of(new RustStruct("test"), new RustStruct("test2")));
        assertEquals(2, vec.size());
        assertEquals("test", vec.get(0));
        assertEquals("test2", vec.get(1));
    }

    @Test
    public void testGetVecValuesNull() {
        var msg =
                assertThrows(NullPointerException.class, () -> NativeStruct.getVecValues(null))
                        .getMessage();
        assertEquals("Null pointer in get_list obj argument", msg);
    }

    @Test
    public void testGetTypeHash() {
        assertDoesNotThrow(NativeStruct::getTypeHash);
    }
}
