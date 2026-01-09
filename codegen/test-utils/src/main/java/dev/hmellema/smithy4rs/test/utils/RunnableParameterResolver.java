/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.test.utils;

import java.util.function.Consumer;
import org.junit.jupiter.api.extension.ExtensionContext;
import org.junit.jupiter.api.extension.ParameterContext;
import org.junit.jupiter.api.extension.ParameterResolutionException;
import software.amazon.smithy.model.node.ObjectNode;

/**
 * Allows a test case to be provided to the test method as a runnable.
 */
@FunctionalInterface
public interface RunnableParameterResolver extends org.junit.jupiter.api.extension.ParameterResolver {
    @Override
    default boolean supportsParameter(
            ParameterContext paramCtx,
            ExtensionContext ignored
    ) throws ParameterResolutionException {
        return Consumer.class.isAssignableFrom(paramCtx.getParameter().getType())
                && paramCtx.getIndex() == 0;
    }

    @Override
    default Object resolveParameter(
            ParameterContext ignoredOne,
            ExtensionContext ignoredTwo
    ) throws ParameterResolutionException {
        return (Consumer<ObjectNode>) this::execute;
    }

    void execute(ObjectNode settings);
}
