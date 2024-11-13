/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as environments from "../../../../../../environments";
import * as core from "../../../../../../core";
import * as Rivet from "../../../../../index";
export declare namespace Lobbies {
    interface Options {
        environment?: core.Supplier<environments.RivetEnvironment | string>;
        token?: core.Supplier<core.BearerToken | undefined>;
        fetcher?: core.FetchFunction;
    }
    interface RequestOptions {
        /** The maximum time to wait for a response in seconds. */
        timeoutInSeconds?: number;
        /** The number of times to retry the request. Defaults to 2. */
        maxRetries?: number;
        /** A hook to abort the request. */
        abortSignal?: AbortSignal;
    }
}
export declare class Lobbies {
    protected readonly _options: Lobbies.Options;
    constructor(_options?: Lobbies.Options);
    /**
     * Marks the current lobby as ready to accept connections. Players will not be able to connect to this lobby until the lobby is flagged as ready.
     * This endpoint requires a [lobby token](/docs/general/concepts/token-types#matchmaker-lobby) for authentication, or a [development namespace token](/docs/general/concepts/token-types#namespace-development) for mock responses. When running on Rivet servers, you can access the given lobby token from the [`RIVET_TOKEN`](/docs/matchmaker/concepts/lobby-env) environment variable.
     *
     * @param {Lobbies.RequestOptions} requestOptions - Request-specific configuration.
     *
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     *
     * @example
     *     await client.matchmaker.lobbies.ready()
     */
    ready(requestOptions?: Lobbies.RequestOptions): Promise<void>;
    /**
     * If `is_closed` is `true`, the matchmaker will no longer route players to the lobby. Players can still
     * join using the /join endpoint (this can be disabled by the developer by rejecting all new connections
     * after setting the lobby to closed).
     * Does not shutdown the lobby.
     *
     * This endpoint requires a [lobby token](/docs/general/concepts/token-types#matchmaker-lobby) for
     * authentication, or a [development namespace token](/docs/general/concepts/token-types#namespace-development)
     * for mock responses. When running on Rivet servers, you can access the given lobby token from the
     * [`RIVET_TOKEN`](/docs/matchmaker/concepts/lobby-env) environment variable.
     *
     * @param {Rivet.matchmaker.SetLobbyClosedRequest} request
     * @param {Lobbies.RequestOptions} requestOptions - Request-specific configuration.
     *
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     *
     * @example
     *     await client.matchmaker.lobbies.setClosed({
     *         isClosed: true
     *     })
     */
    setClosed(request: Rivet.matchmaker.SetLobbyClosedRequest, requestOptions?: Lobbies.RequestOptions): Promise<void>;
    /**
     * Sets the state JSON of the current lobby.
     *
     * This endpoint requires a [lobby token](/docs/general/concepts/token-types#matchmaker-lobby) for
     * authentication, or a [development namespace token](/docs/general/concepts/token-types#namespace-development)
     * for mock responses. When running on Rivet servers, you can access the given lobby token from the
     * [`RIVET_TOKEN`](/docs/matchmaker/concepts/lobby-env) environment variable.
     *
     * @param {unknown} request
     * @param {Lobbies.RequestOptions} requestOptions - Request-specific configuration.
     *
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     *
     * @example
     *     await client.matchmaker.lobbies.setState({
     *         "key": "value"
     *     })
     */
    setState(request?: unknown, requestOptions?: Lobbies.RequestOptions): Promise<void>;
    /**
     * Get the state of any lobby.
     *
     * This endpoint requires a [lobby token](/docs/general/concepts/token-types#matchmaker-lobby) for
     * authentication, or a [development namespace token](/docs/general/concepts/token-types#namespace-development)
     * for mock responses. When running on Rivet servers, you can access the given lobby token from the
     * [`RIVET_TOKEN`](/docs/matchmaker/concepts/lobby-env) environment variable.
     *
     * @param {string} lobbyId
     * @param {Lobbies.RequestOptions} requestOptions - Request-specific configuration.
     *
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     *
     * @example
     *     await client.matchmaker.lobbies.getState("d5e9c84f-c2b2-4bf4-b4b0-7ffd7a9ffc32")
     */
    getState(lobbyId: string, requestOptions?: Lobbies.RequestOptions): Promise<unknown | undefined>;
    /**
     * Finds a lobby based on the given criteria.
     * If a lobby is not found and `prevent_auto_create_lobby` is `false`,
     * a new lobby will be created.
     *
     * When [tokenless authentication](/docs/general/concepts/tokenless-authentication/web) is enabled in
     * your game namespace, this endpoint does not require a token to authenticate. Otherwise, a
     * [development namespace token](/docs/general/concepts/token-types#namespace-development) can be used
     * for mock responses and a [public namespace token](/docs/general/concepts/token-types#namespace-public)
     * can be used for general authentication.
     *
     * @param {Rivet.matchmaker.FindLobbyRequest} request
     * @param {Lobbies.RequestOptions} requestOptions - Request-specific configuration.
     *
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     *
     * @example
     *     await client.matchmaker.lobbies.find({
     *         origin: "string",
     *         gameModes: ["string"],
     *         regions: ["string"],
     *         preventAutoCreateLobby: true,
     *         tags: {
     *             "string": "string"
     *         },
     *         maxPlayers: 1,
     *         captcha: {
     *             hcaptcha: {
     *                 clientResponse: "string"
     *             },
     *             turnstile: {
     *                 clientResponse: "string"
     *             }
     *         },
     *         verificationData: {
     *             "key": "value"
     *         }
     *     })
     */
    find(request: Rivet.matchmaker.FindLobbyRequest, requestOptions?: Lobbies.RequestOptions): Promise<Rivet.matchmaker.FindLobbyResponse>;
    /**
     * Joins a specific lobby.
     * This request will use the direct player count configured for the
     * lobby group.
     *
     * When [tokenless authentication](/docs/general/concepts/tokenless-authentication/web) is enabled in
     * your game namespace, this endpoint does not require a token to authenticate. Otherwise, a
     * [development namespace token](/docs/general/concepts/token-types#namespace-development) can be used
     * for mock responses and a [public namespace token](/docs/general/concepts/token-types#namespace-public)
     * can be used for general authentication.
     *
     * @param {Rivet.matchmaker.JoinLobbyRequest} request
     * @param {Lobbies.RequestOptions} requestOptions - Request-specific configuration.
     *
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     *
     * @example
     *     await client.matchmaker.lobbies.join({
     *         lobbyId: "string",
     *         captcha: {
     *             hcaptcha: {
     *                 clientResponse: "string"
     *             },
     *             turnstile: {
     *                 clientResponse: "string"
     *             }
     *         },
     *         verificationData: {
     *             "key": "value"
     *         }
     *     })
     */
    join(request: Rivet.matchmaker.JoinLobbyRequest, requestOptions?: Lobbies.RequestOptions): Promise<Rivet.matchmaker.JoinLobbyResponse>;
    /**
     * Creates a custom lobby.
     *
     * When [tokenless authentication](/docs/general/concepts/tokenless-authentication/web) is enabled in
     * your game namespace, this endpoint does not require a token to authenticate. Otherwise, a
     * [development namespace token](/docs/general/concepts/token-types#namespace-development) can be used
     * for mock responses and a [public namespace token](/docs/general/concepts/token-types#namespace-public)
     * can be used for general authentication.
     *
     * @param {Rivet.matchmaker.CreateLobbyRequest} request
     * @param {Lobbies.RequestOptions} requestOptions - Request-specific configuration.
     *
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     *
     * @example
     *     await client.matchmaker.lobbies.create({
     *         gameMode: "string",
     *         region: "string",
     *         publicity: Rivet.matchmaker.CustomLobbyPublicity.Public,
     *         tags: {
     *             "string": "string"
     *         },
     *         maxPlayers: 1,
     *         lobbyConfig: {
     *             "key": "value"
     *         },
     *         captcha: {
     *             hcaptcha: {
     *                 clientResponse: "string"
     *             },
     *             turnstile: {
     *                 clientResponse: "string"
     *             }
     *         },
     *         verificationData: {
     *             "key": "value"
     *         }
     *     })
     */
    create(request: Rivet.matchmaker.CreateLobbyRequest, requestOptions?: Lobbies.RequestOptions): Promise<Rivet.matchmaker.CreateLobbyResponse>;
    /**
     * Lists all open lobbies.
     *
     * When [tokenless authentication](/docs/general/concepts/tokenless-authentication/web) is enabled in
     * your game namespace, this endpoint does not require a token to authenticate. Otherwise, a
     * [development namespace token](/docs/general/concepts/token-types#namespace-development) can be used
     * for mock responses and a [public namespace token](/docs/general/concepts/token-types#namespace-public)
     * can be used for general authentication.
     *
     * @param {Rivet.matchmaker.ListLobbiesRequest} request
     * @param {Lobbies.RequestOptions} requestOptions - Request-specific configuration.
     *
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     *
     * @example
     *     await client.matchmaker.lobbies.list({
     *         includeState: true
     *     })
     */
    list(request?: Rivet.matchmaker.ListLobbiesRequest, requestOptions?: Lobbies.RequestOptions): Promise<Rivet.matchmaker.ListLobbiesResponse>;
    protected _getAuthorizationHeader(): Promise<string | undefined>;
}
