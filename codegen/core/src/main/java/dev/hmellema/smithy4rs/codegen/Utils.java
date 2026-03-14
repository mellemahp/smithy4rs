/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen;

import java.net.URL;
import java.util.Objects;
import java.util.Set;
import software.amazon.smithy.codegen.core.ReservedWords;
import software.amazon.smithy.codegen.core.ReservedWordsBuilder;
import software.amazon.smithy.model.loader.Prelude;
import software.amazon.smithy.model.shapes.Shape;
import software.amazon.smithy.model.shapes.ShapeId;
import software.amazon.smithy.model.traits.EnumValueTrait;
import software.amazon.smithy.model.traits.ExamplesTrait;
import software.amazon.smithy.model.traits.HttpApiKeyAuthTrait;
import software.amazon.smithy.model.traits.ReferencesTrait;
import software.amazon.smithy.model.traits.TraitValidatorsTrait;

public final class Utils {
    private static final URL RUST_RESERVED_WORDS = Objects.requireNonNull(
            Utils.class.getResource("rust-reserved-words.txt"));
    public static final ReservedWords SHAPE_ESCAPER = new ReservedWordsBuilder()
            .loadCaseInsensitiveWords(RUST_RESERVED_WORDS, word -> word + "Shape")
            .build();
    public static final ReservedWords MEMBER_ESCAPER = new ReservedWordsBuilder()
            .loadCaseInsensitiveWords(RUST_RESERVED_WORDS, word -> word + "Member")
            .build();
    private static final String DEFAULT_CRATE_IDENT = "smithy4rs_core";
    private static final String CRATE = "crate";
    public static final String DELIM = "::";

    private static final Set<ShapeId> SKIPPED_SHAPES = Set.of(
            // We do not support deprecated enum trait.
            ShapeId.from("smithy.api#enum"),
            EnumValueTrait.ID,
            // TODO: Wrappers on nested collections is broken ATM.
            ReferencesTrait.ID,
            TraitValidatorsTrait.ID,
            ExamplesTrait.ID,
            ShapeId.from("smithy.api#Example"),
            // TODO: Fails due to clone impl
            ShapeId.from("smithy.api#ExampleError"),
            // TODO: Needs member name escaping
            HttpApiKeyAuthTrait.ID);

    public static boolean inCore() {
        return EnvironmentVariable.IN_SMITH4RS_CORE.isSet();
    }

    public static String crateIdent() {
        return inCore() ? CRATE : DEFAULT_CRATE_IDENT;
    }

    public static boolean shouldInclude(Shape s) {
        // TODO: Is there another way to detect current shapes to generate?
        // Only filter prelude shapes if we are not in core.
        if (SKIPPED_SHAPES.contains(s.getId())) {
            return false;
        }
        return Utils.inCore() || !Prelude.isPreludeShape(s);
    }

    private Utils() {
        // Utility class should not be instantiated
    }
}
