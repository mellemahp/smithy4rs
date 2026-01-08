package dev.hmellema.smithy4rs.test.utils;


import software.amazon.smithy.build.PluginContext;
import software.amazon.smithy.build.SmithyBuildPlugin;

public class TestGenerationPlugin implements SmithyBuildPlugin {
    @Override
    public String getName() {
        return "test-codegen";
    }

    @Override
    public void execute(PluginContext context) {
        var rustString = """
                struct B {
                    a: String,
                    b: i32
                }
                """;
        context.getFileManifest().writeFile("smithy-generated.rs", rustString);
    }
}
