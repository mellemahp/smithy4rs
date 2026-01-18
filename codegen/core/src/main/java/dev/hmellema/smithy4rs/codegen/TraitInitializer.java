/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen;

import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import software.amazon.smithy.model.traits.Trait;

/**
 * Writes an initializer for a trait when adding that trait to a `Schema` definition.
 *
 * <p>{@code TraitInitializer} implementations can be added to a {@link RustCodegenIntegration} to customize the way
 * in which traits are initialized in a Schema definition. Custom initializers are useful to improve the performance of
 * initializing complex traits in generated code.
 *
 * <p>For example, in the following schema definition:
 * {@snippet lang=rust :
 * smithy!("com.example#Shape": {
 *    @ExampleTrait::builder().value("a").build();
 *    string Shape
 * });
 * }
 * The initializer for the `ExampleTrait` is {@code ExampleTrait::builder().value("a").build()}.
 *
 * <p>The following initializers are provided by default by the "core" integration:
 * <ul>
 *  <li>{@link  software.amazon.smithy.model.traits.AnnotationTrait}</li>
 *  <li>{@link software.amazon.smithy.model.traits.StringTrait}</li>
 *  <li>{@link software.amazon.smithy.model.traits.StringListTrait}</li>
 *  <li>Catch-all for {@link software.amazon.smithy.model.traits.Trait}</li>
 * </ul>
 *
 * <p>Custom traits are automatically supported by the catch-all initializer. The catch-all initializer uses the
 * {@code TraitService} service provider interface to identify the correct trait provider class for a given trait ID.
 * The trait is then initialized using the `DynamicTrait` struct.
 */
public interface TraitInitializer<T extends Trait> {
    Class<T> traitClass();

    /**
     * Write the trait initializer to the provided writer.
     *
     * @param writer writer to write intializer to
     * @param context code generation context used to resolve trait symbol
     * @param trait trait to initialize
     */
    void write(RustWriter writer, CodeGenerationContext context, T trait);

    /**
     * If this initializer is compatible with the provided trait.
     *
     * @param context code generation context that can be used to resolve a trait mapping.
     * @param trait trait to check
     */
    default boolean isIntercepted(CodeGenerationContext context, Trait trait) {
        return context.traitMapping(trait) != null;
    }
}
