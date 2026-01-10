/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen;

/**
 * The edition of the {@code smithy4rs } code generator to use for code generation.
 *
 * <p>Editions can automatically enable and disable feature-gates in the generator.
 *
 * @see <a href="https://smithy.io/2.0/guides/building-codegen/configuring-the-generator.html#edition">Smithy code generator editions</a>
 */
public enum CodegenEdition {
    V2026,
    LATEST
}
