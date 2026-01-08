package dev.hmellema.smithy4rs.codegen;

import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import java.util.List;
import software.amazon.smithy.build.FileManifest;
import software.amazon.smithy.codegen.core.CodegenContext;
import software.amazon.smithy.codegen.core.SymbolProvider;
import software.amazon.smithy.codegen.core.WriterDelegator;
import software.amazon.smithy.model.Model;

public class CodeGenerationContext implements CodegenContext<RustCodegenSettings, RustWriter, RustCodegenIntegration> {
    @Override
    public Model model() {
        return null;
    }

    @Override
    public RustCodegenSettings settings() {
        return null;
    }

    @Override
    public SymbolProvider symbolProvider() {
        return null;
    }

    @Override
    public FileManifest fileManifest() {
        return null;
    }

    @Override
    public WriterDelegator writerDelegator() {
        return null;
    }

    @Override
    public List integrations() {
        return List.of();
    }
}
