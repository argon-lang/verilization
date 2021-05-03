package dev.argon.verilization.runtime;

import java.util.function.Function;
import java.util.stream.Collectors;
import java.util.List;
import java.util.Optional;

public abstract class Util {
    private Util() {}

    public static <T, U> Function<List<T>, List<U>> mapList(Function<T, U> f) {
        return x -> x.stream().map(f).collect(Collectors.toList());
    }

    public static <T, U> Function<Optional<T>, Optional<U>> mapOption(Function<T, U> f) {
        return x -> x.map(f);
    }
}
