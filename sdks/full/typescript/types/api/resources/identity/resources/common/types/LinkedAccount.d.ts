/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as Rivet from "../../../../../index";
/**
 * A union representing an identity's linked accounts.
 */
export interface LinkedAccount {
    email?: Rivet.identity.EmailLinkedAccount;
    defaultUser?: boolean;
}
