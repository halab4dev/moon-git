import { beginSignIn, findAccount, finishSignIn } from "../api/account";
import { toAppError } from "../api/errors";
import { useDialogStore } from "../stores/dialogs";

// Step 13b: run a git command that talks to a server; if the server wants a
// login that MoonGit doesn't have (or refuses the saved one), ask for it in
// the "Sign in" dialog, save it, and run the command again.
//
//   withSignIn(() => pushRepo(…))
//     ├─ works                         → done (no dialog)
//     └─ "authRequired" for github.com → Sign in dialog
//          Sign In → beginSignIn (saved, see storage/accounts.rs) → run again
//             ├─ works                → finishSignIn(accepted) → dialog closes
//             ├─ refused again        → finishSignIn(refused: undone) → message in the dialog
//             └─ another error        → login kept (the server didn't refuse it);
//                                       dialog closes, the error goes to the caller
//          Cancel → the first error goes to the caller, as before step 13b

export function withSignIn<T>(action: () => Promise<T>): Promise<T> {
  return action().catch(async (err: unknown) => {
    const first = toAppError(err);
    if (first.kind !== "authRequired" || !first.server) {
      throw first;
    }
    return askAndRetry(first.server, first.message, action, first);
  });
}

async function askAndRetry<T>(server: string, reason: string, action: () => Promise<T>, first: unknown): Promise<T> {
  // Prefill the user name of a saved account (its token was just refused).
  const saved = await findAccount(server).catch(() => null);
  return new Promise<T>((resolve, reject) => {
    useDialogStore().askSignIn({
      server,
      username: saved?.username ?? "",
      reason,
      cancel: () => reject(first),
      submit: async (username, token) => {
        let id: string;
        try {
          id = await beginSignIn(server, username, token);
        } catch (err) {
          return toAppError(err).message;
        }
        try {
          const result = await action();
          await finishSignIn(id, true);
          resolve(result);
          return null;
        } catch (err) {
          const error = toAppError(err);
          if (error.kind === "authRequired") {
            await finishSignIn(id, false);
            return `${error.message} Check the user name and the token, then try again.`;
          }
          // Not about the login (e.g. the push was rejected): keep it.
          await finishSignIn(id, true);
          reject(error);
          return null;
        }
      },
    });
  });
}
