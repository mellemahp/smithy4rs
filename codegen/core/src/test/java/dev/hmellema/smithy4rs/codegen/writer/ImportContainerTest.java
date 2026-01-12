/*
 * Copyright Hunter Mellema & Hayden Baker. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
package dev.hmellema.smithy4rs.codegen.writer;

import org.junit.jupiter.api.Test;
import software.amazon.smithy.codegen.core.Symbol;

class ImportContainerTest {
    @Test
    void importsPrintCorrectly() {
        var container = new RustImportContainer();
        var symbolA = Symbol.builder()
                .name("A")
                .namespace("test::inner", "::")
                .build();
        var symbolB = Symbol.builder()
                .name("B")
                .namespace("test::inner", "::")
                .build();
        var symbolC = Symbol.builder()
                .name("C")
                .namespace("test::inner::other", "::")
                .build();
        var symbolD = Symbol.builder()
                .name("D")
                .namespace("test::inner::also", "::")
                .build();
        var symbolE = Symbol.builder()
                .name("E")
                .namespace("test::inner::also", "::")
                .build();
        var symbolF = Symbol.builder()
                .name("F")
                .namespace("other::thingy::stuff", "::")
                .build();
        container.importSymbol(symbolA);
        container.importSymbol(symbolB);
        container.importSymbol(symbolC);
        container.importSymbol(symbolD);
        container.importSymbol(symbolE);
        container.importSymbol(symbolF);
        System.out.println(container);
    }
}
