package dev.hmellema.smithy4rs.codegen.writer;

import software.amazon.smithy.codegen.core.SymbolWriter;

public class RustWriter extends SymbolWriter<RustWriter, RustImportContainer> {
    /**
     * @param importContainer Container used to persist and filter imports.
     */
    public RustWriter(RustImportContainer importContainer) {
        super(importContainer);
    }
}
