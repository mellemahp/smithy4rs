/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen;

import software.amazon.smithy.utils.SmithyInternalApi;

/**
 * Environment variables used by the code generator
 */
@SmithyInternalApi
enum EnvironmentVariable {
    /**
     * If the generator is running in the `core` crate. True if present
     * <p>
     * WARNING: Users should not set this variable.
     */
    IN_SMITH4RS_CORE;

    /**
     * Returns true if the system property or environment variables is set.
     *
     * @return Returns true if set.
     */
    public boolean isSet() {
        return get() != null;
    }

    /**
     * Gets the system property or the environment variable for the property, in that order.
     *
     * @return Returns the found system property or environment variable or null.
     */
    public String get() {
        String value = System.getProperty(toString());
        if (value == null) {
            value = System.getenv(toString());
        }
        return value;
    }
}
