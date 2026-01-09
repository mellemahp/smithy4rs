package dev.hmellema.smithy4rs.codegen;

import dev.hmellema.smithy4rs.codegen.generators.ListGenerator;
import dev.hmellema.smithy4rs.codegen.generators.MapGenerator;
import dev.hmellema.smithy4rs.codegen.generators.ScalarSchemaGenerator;
import dev.hmellema.smithy4rs.codegen.generators.StructureGenerator;
import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import software.amazon.smithy.codegen.core.SymbolProvider;
import software.amazon.smithy.codegen.core.WriterDelegator;
import software.amazon.smithy.codegen.core.directed.CreateContextDirective;
import software.amazon.smithy.codegen.core.directed.CreateSymbolProviderDirective;
import software.amazon.smithy.codegen.core.directed.CustomizeDirective;
import software.amazon.smithy.codegen.core.directed.DirectedCodegen;
import software.amazon.smithy.codegen.core.directed.GenerateEnumDirective;
import software.amazon.smithy.codegen.core.directed.GenerateErrorDirective;
import software.amazon.smithy.codegen.core.directed.GenerateIntEnumDirective;
import software.amazon.smithy.codegen.core.directed.GenerateListDirective;
import software.amazon.smithy.codegen.core.directed.GenerateMapDirective;
import software.amazon.smithy.codegen.core.directed.GenerateServiceDirective;
import software.amazon.smithy.codegen.core.directed.GenerateStructureDirective;
import software.amazon.smithy.codegen.core.directed.GenerateUnionDirective;

public class TestRustCodegen implements
        DirectedCodegen<CodeGenerationContext, RustCodegenSettings, RustCodegenIntegration> {
    public CodeGenerationContext context;

    @Override
    public CodeGenerationContext createContext(CreateContextDirective<RustCodegenSettings, RustCodegenIntegration> directive) {
        return new CodeGenerationContext(
                directive.model(),
                directive.settings(),
                directive.symbolProvider(),
                directive.fileManifest(),
                new WriterDelegator<>(
                        directive.fileManifest(),
                        directive.symbolProvider(),
                        new RustWriter.Factory(directive.settings())),
                directive.integrations()
        );
    }

    @Override
    public SymbolProvider createSymbolProvider(
            CreateSymbolProviderDirective<RustCodegenSettings> directive
    ) {
        return new RustSymbolProvider(directive.model());
    }

    @Override
    public void generateService(GenerateServiceDirective<CodeGenerationContext, RustCodegenSettings> directive) {
        return;
    }

    @Override
    public void generateStructure(GenerateStructureDirective<CodeGenerationContext, RustCodegenSettings> directive) {
        new StructureGenerator().accept(directive);
    }

    @Override
    public void generateError(GenerateErrorDirective<CodeGenerationContext, RustCodegenSettings> directive) {

    }

    @Override
    public void generateUnion(GenerateUnionDirective<CodeGenerationContext, RustCodegenSettings> directive) {

    }

    @Override
    public void generateList(GenerateListDirective<CodeGenerationContext, RustCodegenSettings> directive) {
        new ListGenerator().accept(directive);
    }

    @Override
    public void generateMap(GenerateMapDirective<CodeGenerationContext, RustCodegenSettings> directive) {
        new MapGenerator().accept(directive);
    }

    @Override
    public void generateEnumShape(GenerateEnumDirective<CodeGenerationContext, RustCodegenSettings> directive) {

    }

    @Override
    public void generateIntEnumShape(GenerateIntEnumDirective<CodeGenerationContext, RustCodegenSettings> directive) {

    }

    @Override
    public void customizeBeforeIntegrations(CustomizeDirective<CodeGenerationContext, RustCodegenSettings> directive) {
        new ScalarSchemaGenerator().accept(directive);
    }
}
