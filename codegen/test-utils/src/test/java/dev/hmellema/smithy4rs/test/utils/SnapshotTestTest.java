/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.test.utils;

import java.util.function.Consumer;
import software.amazon.smithy.model.node.ObjectNode;

public class SnapshotTestTest {
    private static final ObjectNode TEST_SETTINGS = ObjectNode.builder().build();

    @RustCodegenTest("test-codegen")
    void execute(Consumer<ObjectNode> test) {
        test.accept(TEST_SETTINGS);
    }
}
