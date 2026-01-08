/*
 * Copyright Scaffold Software LLC. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.test.utils;

/**
 * Indicates that Translation snapshot tests could not be loaded or executed correctly.
 */
public class RustCodegenTestException extends RuntimeException {
    public RustCodegenTestException(String message) {
        super(message);
    }

    public RustCodegenTestException(String message, Throwable cause) {
        super(message, cause);
    }
}
