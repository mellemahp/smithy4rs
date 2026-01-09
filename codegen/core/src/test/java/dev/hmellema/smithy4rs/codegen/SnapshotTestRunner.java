package dev.hmellema.smithy4rs.codegen;

import dev.hmellema.smithy4rs.test.utils.RustCodegenTest;
import java.util.function.Consumer;
import software.amazon.smithy.model.node.ObjectNode;

final class SnapshotTestRunner {
    private static final ObjectNode TEST_SETTINGS = ObjectNode.builder().build();

    @RustCodegenTest("test-rust-codegen-core")
    void execute(Consumer<ObjectNode> test) {
        test.accept(TEST_SETTINGS);
    }
}
