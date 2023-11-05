package com.github.markusjx.example;

import com.github.markusjx.generated.StructUsingTrait;
import java.util.concurrent.atomic.AtomicBoolean;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;

public class TestInterface {
    @Test
    public void testUseApplyString() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                "test from java",
                StructUsingTrait.useApplyString(
                        val -> {
                            called.set(true);
                            return val + " from java";
                        }));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyStringNull() {
        var message =
                Assertions.assertThrows(
                                NullPointerException.class,
                                () -> StructUsingTrait.useApplyString(null))
                        .getMessage();
        Assertions.assertEquals("Null pointer in call_method obj argument", message);
    }

    @Test
    public void testUseApplyStringOpt() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                "test from java",
                StructUsingTrait.useApplyStringOpt(
                        val -> {
                            called.set(true);
                            return val + " from java";
                        }));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyStringVec() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                "test from java",
                String.join(
                        " ",
                        StructUsingTrait.useApplyStringVec(
                                val -> {
                                    called.set(true);
                                    val.add("from java");
                                    return val;
                                })));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyStringVecOpt() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                "test from java",
                String.join(
                        " ",
                        StructUsingTrait.useApplyStringVecOpt(
                                val -> {
                                    called.set(true);
                                    val.add("from java");
                                    return val;
                                })));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyStringHashmap() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                "test from java",
                String.join(
                        " ",
                        StructUsingTrait.useApplyStringHashmap(
                                        val -> {
                                            called.set(true);
                                            val.put("from java", "from java");
                                            return val;
                                        })
                                .keySet()));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyStringHashmapOpt() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                "test from java",
                String.join(
                        " ",
                        StructUsingTrait.useApplyStringHashmapOpt(
                                        val -> {
                                            called.set(true);
                                            val.put("from java", "from java");
                                            return val;
                                        })
                                .keySet()));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyInt() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                2,
                StructUsingTrait.useApplyInt(
                        val -> {
                            called.set(true);
                            return val + 1;
                        }));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyIntOpt() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                2,
                StructUsingTrait.useApplyIntOpt(
                        val -> {
                            called.set(true);
                            return val + 1;
                        }));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyLong() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                2L,
                StructUsingTrait.useApplyLong(
                        val -> {
                            called.set(true);
                            return val + 1;
                        }));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyLongOpt() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                2L,
                StructUsingTrait.useApplyLongOpt(
                        val -> {
                            called.set(true);
                            return val + 1;
                        }));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyFloat() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                2.0f,
                StructUsingTrait.useApplyFloat(
                        val -> {
                            called.set(true);
                            return val + 1;
                        }));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyFloatOpt() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                2.0f,
                StructUsingTrait.useApplyFloatOpt(
                        val -> {
                            called.set(true);
                            return val + 1;
                        }));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyDouble() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                2.0,
                StructUsingTrait.useApplyDouble(
                        val -> {
                            called.set(true);
                            return val + 1;
                        }));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyDoubleOpt() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                2.0,
                StructUsingTrait.useApplyDoubleOpt(
                        val -> {
                            called.set(true);
                            return val + 1;
                        }));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyBool() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertFalse(
                StructUsingTrait.useApplyBool(
                        val -> {
                            called.set(true);
                            return !val;
                        }));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyBoolOpt() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                false,
                StructUsingTrait.useApplyBoolOpt(
                        val -> {
                            called.set(true);
                            return !val;
                        }));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyByte() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                (byte) 2,
                StructUsingTrait.useApplyByte(
                        val -> {
                            called.set(true);
                            return (byte) (val + 1);
                        }));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyByteOpt() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                (byte) 2,
                StructUsingTrait.useApplyByteOpt(
                        val -> {
                            called.set(true);
                            return (byte) (val + 1);
                        }));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyChar() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                'b',
                StructUsingTrait.useApplyChar(
                        val -> {
                            called.set(true);
                            return (char) (val + 1);
                        }));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyCharOpt() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                'b',
                StructUsingTrait.useApplyCharOpt(
                        val -> {
                            called.set(true);
                            return (char) (val + 1);
                        }));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyShort() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                (short) 2,
                StructUsingTrait.useApplyShort(
                        val -> {
                            called.set(true);
                            return (short) (val + 1);
                        }));
        Assertions.assertTrue(called.get());
    }

    @Test
    public void testUseApplyShortOpt() throws Exception {
        AtomicBoolean called = new AtomicBoolean(false);

        Assertions.assertEquals(
                (short) 2,
                StructUsingTrait.useApplyShortOpt(
                        val -> {
                            called.set(true);
                            return (short) (val + 1);
                        }));
        Assertions.assertTrue(called.get());
    }
}
