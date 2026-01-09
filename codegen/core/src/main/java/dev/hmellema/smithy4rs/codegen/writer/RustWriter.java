package dev.hmellema.smithy4rs.codegen.writer;

import dev.hmellema.smithy4rs.codegen.RustCodegenSettings;
import dev.hmellema.smithy4rs.codegen.SymbolProperties;
import java.util.function.BiFunction;
import software.amazon.smithy.codegen.core.Symbol;
import software.amazon.smithy.codegen.core.SymbolReference;
import software.amazon.smithy.codegen.core.SymbolWriter;

public class RustWriter extends SymbolWriter<RustWriter, RustImportContainer> {
    private final RustCodegenSettings settings;
    private final String namespace;
    private final String filename;

    public RustWriter(RustCodegenSettings settings, String namespace, String filename) {
        super(new RustImportContainer());
        this.settings = settings;
        this.namespace = namespace;
        this.filename = filename;

        // Ensure extraneous white space is trimmed
        trimBlankLines();
        trimTrailingSpaces();

        // Formatters
        putFormatter('T', new RustTypeFormatter());
        putFormatter('I', new RustIdentifierFormatter());
    }

    public static final class Factory implements SymbolWriter.Factory<RustWriter> {

        private final RustCodegenSettings settings;

        /**
         * @param settings The python plugin settings.
         */
        public Factory(RustCodegenSettings settings) {
            this.settings = settings;
        }

        @Override
        public RustWriter apply(String filename, String namespace) {
            return new RustWriter(settings, namespace, filename);
        }
    }

    /**
     * Implements a formatter for {@code $T} that formats Rust types.
     */
    private final class RustTypeFormatter implements BiFunction<Object, String, String> {
        @Override
        public String apply(Object type, String indent) {
            Symbol typeSymbol = getTypeSymbol(type, 'T');

            if (typeSymbol.getReferences().isEmpty()) {
                return getPlaceholder(typeSymbol);
            }

            // Add type references as type references (ex. `IndexMap<KeyType, ValueType>`)
            putContext("refs", typeSymbol.getReferences());
            String output = format(
                    "$L<${#refs}${value:T}${^key.last}, ${/key.last}${/refs}>",
                    getPlaceholder(typeSymbol));
            removeContext("refs");
            return output;
        }

        private String getPlaceholder(Symbol symbol) {
            // TODO: Implement de-duplication
            return symbol.getName();
        }
    }

    /**
     * Implements a formatter for {@code $I} that formats Rust Schema identifiers.
     */
    private final class RustIdentifierFormatter implements BiFunction<Object, String, String> {
        @Override
        public String apply(Object type, String indent) {
            Symbol typeSymbol = getTypeSymbol(type, 'I');
            return typeSymbol.expectProperty(SymbolProperties.SCHEMA_IDENT);
        }

        private String getPlaceholder(Symbol symbol) {
            // TODO: Implement de-duplication
            return symbol.getName();
        }
    }

    private static Symbol getTypeSymbol(Object type, char formatChar) {
        return switch (type) {
            case Symbol s -> s;
            case SymbolReference r -> r.getSymbol();
            case null, default -> throw new IllegalArgumentException(
                    "Invalid type provided for " + formatChar + ". Expected a Symbol"
                            + " but found: `" + type + "`.");
        };
    }

}
