import { InnerError, OuterError, Result } from "@/models/result";
import {GameSession} from "@/models/game-session";

export type ApiResult<T> = Result<Result<T, InnerError>, OuterError>;

export type ListSessionsResult = ApiResult<string[]>;

export type CreatedNewSessionResult = ApiResult<string>;

export type JoinSessionResult = ApiResult<void>;

export type GetGameSessionResult = ApiResult<GameSession>;

export type MakeChessMoveResult = ApiResult<void>;
