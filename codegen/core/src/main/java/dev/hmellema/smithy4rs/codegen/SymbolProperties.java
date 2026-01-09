package dev.hmellema.smithy4rs.codegen;

import software.amazon.smithy.codegen.core.Property;

/**
 * Properties that may be added to symbols by {@code smithy4rs}.
 */
public final class SymbolProperties {
    /**
     * Indicates if a symbol represents a Rust macro type.
     */
    public static final Property<Boolean> IS_MACRO = Property.named("is-macro");

    /**
     * Indicates the name of the static SCHEMA field corresponding to this type
     */
    public static final Property<String> SCHEMA_IDENT = Property.named("schema-name");

    private SymbolProperties() {}
}
