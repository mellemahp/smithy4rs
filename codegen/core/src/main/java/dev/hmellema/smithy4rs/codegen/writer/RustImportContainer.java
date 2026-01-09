/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.writer;

import software.amazon.smithy.codegen.core.ImportContainer;
import software.amazon.smithy.codegen.core.Symbol;

public class RustImportContainer implements ImportContainer {
    @Override
    public void importSymbol(Symbol symbol, String alias) {
        // TODO document why this method is empty
    }
}
