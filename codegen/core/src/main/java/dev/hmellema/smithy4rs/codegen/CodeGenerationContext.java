/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen;

import dev.hmellema.smithy4rs.codegen.writer.RustWriter;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import software.amazon.smithy.build.FileManifest;
import software.amazon.smithy.codegen.core.CodegenContext;
import software.amazon.smithy.codegen.core.Symbol;
import software.amazon.smithy.codegen.core.SymbolProvider;
import software.amazon.smithy.codegen.core.WriterDelegator;
import software.amazon.smithy.model.Model;
import software.amazon.smithy.model.shapes.ShapeId;
import software.amazon.smithy.model.traits.Trait;

public final class CodeGenerationContext
        implements CodegenContext<RustCodegenSettings, RustWriter, RustCodegenIntegration> {
    private final Model model;
    private final RustCodegenSettings settings;
    private final SymbolProvider symbolProvider;
    private final FileManifest fileManifest;
    private final WriterDelegator<RustWriter> writerDelegator;
    private final List<RustCodegenIntegration> integrations;
    private final List<TraitInitializer<?>> traitInitializers;
    private final Map<ShapeId, Symbol> traitMappings;

    public CodeGenerationContext(
            Model model,
            RustCodegenSettings settings,
            SymbolProvider symbolProvider,
            FileManifest fileManifest,
            List<RustCodegenIntegration> integrations
    ) {
        this.model = model;
        this.settings = settings;
        this.symbolProvider = symbolProvider;
        this.fileManifest = fileManifest;
        this.writerDelegator = new WriterDelegator<>(
                this.fileManifest,
                this.symbolProvider,
                new RustWriter.Factory(settings));
        this.integrations = integrations;
        this.traitMappings = collectTraitMappings(integrations);
        this.traitInitializers = collectTraitInitializers(integrations);
    }

    @Override
    public Model model() {
        return this.model;
    }

    @Override
    public RustCodegenSettings settings() {
        return this.settings;
    }

    @Override
    public SymbolProvider symbolProvider() {
        return this.symbolProvider;
    }

    @Override
    public FileManifest fileManifest() {
        return this.fileManifest;
    }

    @Override
    public WriterDelegator<RustWriter> writerDelegator() {
        return this.writerDelegator;
    }

    @Override
    public List<RustCodegenIntegration> integrations() {
        return this.integrations;
    }

    private static Map<ShapeId, Symbol> collectTraitMappings(List<RustCodegenIntegration> integrations) {
        Map<ShapeId, Symbol> result = new HashMap<>();
        for (var integration : integrations) {
            result.putAll(integration.traitMappings());
        }
        return result;
    }

    // TODO: Docs
    public Symbol traitMapping(Trait trait) {
        return traitMappings.get(trait.toShapeId());
    }

    private static List<TraitInitializer<?>> collectTraitInitializers(List<RustCodegenIntegration> integrations) {
        List<TraitInitializer<?>> initializers = new ArrayList<>();
        for (var integration : integrations) {
            initializers.addAll(integration.traitInitializers());
        }
        return initializers;
    }

    /**
     * Gets the {@link TraitInitializer} for a given trait.
     *
     * <p>This implemenetation is copied from {@code smithy-java} trait initialization
     *
     * @param trait trait to get initializer for.
     * @return Trait initializer for trait class.
     * @throws IllegalArgumentException if no initializer can be found for a trait.
     */
    @SuppressWarnings("unchecked")
    public <T extends Trait> TraitInitializer<T> getInitializer(T trait) {
        for (var initializer : traitInitializers) {
            if (initializer.traitClass().isInstance(trait) && initializer.isIntercepted(this, trait)) {
                return (TraitInitializer<T>) initializer;
            }
        }
        throw new IllegalArgumentException("Could not find initializer for " + trait);
    }
}
