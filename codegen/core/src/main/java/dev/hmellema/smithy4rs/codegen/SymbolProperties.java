/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen;

import software.amazon.smithy.codegen.core.Property;
import software.amazon.smithy.codegen.core.Symbol;

/**
 * Properties that may be added to symbols by {@code smithy4rs}.
 */
public final class SymbolProperties {
    /**
     * Symbol to use for the schema of a shape
     */
    public static final Property<Symbol> SCHEMA_SYMBOL = Property.named("schema-symbol");

    private SymbolProperties() {}
}
