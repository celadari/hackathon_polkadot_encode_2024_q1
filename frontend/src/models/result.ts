

export type Ok<T> = { ok: T };
export type Err<E> = { err: E };
export type Result<T, E> = Ok<T> | Err<E>;

// Define specific error types if known
export type OuterError = string;
export type InnerError = string;