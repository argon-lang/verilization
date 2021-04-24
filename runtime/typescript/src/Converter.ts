
export interface Converter<A, B> {
    convert(prev: A): B;
}

export class IdentityConverter<A> implements Converter<A, A> {
    convert(prev: A): A {
        return prev;
    }
}

export namespace Converter {
    export function identity<A>(): Converter<A, A> {
        return new IdentityConverter<A>();
    }
}
