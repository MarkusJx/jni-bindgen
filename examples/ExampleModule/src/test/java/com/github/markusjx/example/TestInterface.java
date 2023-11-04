package com.github.markusjx.example;

import com.github.markusjx.generated.StructUsingTrait;
import java.util.concurrent.atomic.AtomicBoolean;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;

public class TestInterface {
    @Test
    public void testUseInterface() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                "test from java",
                StructUsingTrait.useTrait(
                        val -> {
                            called.set(true);
                            return val + " from java";
                        }));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseInterfaceNull() {
        var message =
                Assertions.assertThrows(
                                NullPointerException.class, () -> StructUsingTrait.useTrait(null))
                        .getMessage();
        Assertions.assertEquals("Null pointer in call_method obj argument", message);
    }
}
