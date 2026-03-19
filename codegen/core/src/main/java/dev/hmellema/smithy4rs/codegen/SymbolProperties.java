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

    /**
     * Symbol to use for the inner type of wrapper
     */
    public static final Property<Symbol> INNER = Property.named("inner-symbol");

    /**
     * Represents a trait object
     */
    public static final Property<Boolean> IS_DYN = Property.named("dyn");

    /**
     * A standalone type with no builder
     */
    public static final Property<Boolean> NO_BUILDER = Property.named("no-builder");

    /**
     * If a member is required
     */
    public static final Property<Boolean> REQUIRED = Property.named("required");

    /**
     * If a member has a default
     */
    public static final Property<Boolean> HAS_DEFAULT = Property.named("has-default");

    private SymbolProperties() {}
}
